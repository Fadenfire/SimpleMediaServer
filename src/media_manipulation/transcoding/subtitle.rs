use crate::media_manipulation::media_utils;
use crate::media_manipulation::media_utils::in_memory_muxer::InMemoryMuxer;
use crate::media_manipulation::media_utils::{av_error, check_alloc, MILLIS_TIME_BASE};
use anyhow::{anyhow, Context};
use bytes::Bytes;
use ffmpeg_next::codec::Id;
use ffmpeg_next::media::Type;
use ffmpeg_next::{codec, encoder, format, Discard, Packet, Rescale, Subtitle};
use ffmpeg_sys_next::{av_mallocz, avcodec_encode_subtitle, AV_TIME_BASE_Q};
use std::ffi::c_int;
use std::path::PathBuf;

pub fn transcode_subtitle_to_webvtt(media_path: PathBuf, stream_index: usize) -> anyhow::Result<Bytes> {
	let mut demuxer = format::input(&media_path).context("Opening video file")?;
	let mut muxer = InMemoryMuxer::new("webvtt").context("Opening output")?;
	
	// Discard all packets except for the subtitle stream
	media_utils::discard_all_but_one(&mut demuxer, stream_index, Discard::None);
	
	// Get in stream
	
	let in_stream = demuxer.stream(stream_index).context("Could not find subtitle stream")?;
	let in_time_base = in_stream.time_base();
	
	if in_stream.parameters().medium() != Type::Subtitle {
		return Err(anyhow!("Given stream is not a subtitle stream"));
	}
	
	// Create decoder
	
	let mut decoder = codec::context::Context::from_parameters(in_stream.parameters())?
		.decoder()
		.subtitle()?;
	
	decoder.set_packet_time_base(in_time_base);
	
	// Create encoder
	
	let webvtt_codec = encoder::find(Id::WEBVTT).unwrap();
	
	let mut encoder = codec::context::Context::new_with_codec(webvtt_codec)
		.encoder()
		.subtitle()?;
	
	encoder.set_time_base(AV_TIME_BASE_Q);
	
	unsafe {
		if !(*decoder.as_ptr()).subtitle_header.is_null() {
			let subtitle_header_size = (*decoder.as_ptr()).subtitle_header_size;
			// Allocate size + 1 as some encoders expect subtitle_header to be null terminated
			let subtitle_header_buf = check_alloc(av_mallocz((subtitle_header_size + 1) as usize))?.cast();
			
			std::ptr::copy(
				(*decoder.as_ptr()).subtitle_header,
				subtitle_header_buf,
				subtitle_header_size as usize
			);
			
			(*encoder.as_mut_ptr()).subtitle_header = subtitle_header_buf;
			(*encoder.as_mut_ptr()).subtitle_header_size = subtitle_header_size;
		}
	}
	
	let mut encoder = encoder.open_as(webvtt_codec).context("Opening encoder")?;
	
	// Add output stream
	
	let mut out_stream = muxer.add_stream(webvtt_codec).context("Adding output stream")?;
	out_stream.set_parameters(&encoder);
	
	let out_stream_index = out_stream.index();
	let out_time_base = MILLIS_TIME_BASE;
	
	// Begin transcoding
	
	muxer.write_header().context("Writing header")?;
	
	let mut out_buf = vec![0; 1024 * 1024];
	
	for (stream, packet) in demuxer.packets() {
		if stream.index() == stream_index {
			let mut subtitle = Subtitle::new();
			let success = decoder.decode(&packet, &mut subtitle).context("Decoding subtitle")?;
			
			if success {
				// Subtitle PTS is always in AV_TIME_BASE
				let mut sub_pts = subtitle.pts()
					.ok_or_else(|| anyhow!("Decoded subtitle doesn't have a PTS"))?;
				
				// Subtitle start and end times are always in milliseconds
				sub_pts += subtitle.start().rescale(MILLIS_TIME_BASE, AV_TIME_BASE_Q);
				
				let sub_duration = subtitle.end() - subtitle.start();
				
				subtitle.set_pts(Some(sub_pts));
				subtitle.set_end(sub_duration);
				subtitle.set_start(0);
				
				let bytes_written = unsafe {
					av_error(avcodec_encode_subtitle(
						encoder.as_mut_ptr(),
						out_buf.as_mut_ptr(),
						out_buf.len() as c_int,
						subtitle.as_ptr(),
					)).context("Encoding subtitle")?
				};
				
				let packet_pts = sub_pts.rescale(AV_TIME_BASE_Q, out_time_base);
				
				let mut out_packet = Packet::copy(&out_buf[..bytes_written as usize]);
				out_packet.set_time_base(out_time_base);
				out_packet.set_stream(out_stream_index);
				out_packet.set_pts(Some(packet_pts));
				out_packet.set_dts(Some(packet_pts));
				out_packet.set_duration(sub_duration.rescale(MILLIS_TIME_BASE, out_time_base));
				
				out_packet.write_interleaved(&mut muxer).context("Writing packet")?;
			}
		}
	}
	
	muxer.write_trailer().context("Writing trailer")?;
	
	Ok(muxer.into_output_buffer().into())
}