use std::ffi::CString;
use std::ops::Range;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use bytes::Bytes;
use ffmpeg_next::{Dictionary, encoder, format, media, Rescale, rescale};
use ffmpeg_next::codec::Id;
use ffmpeg_sys_next::av_opt_set;

use crate::media_manipulation::backends::BackendFactory;
use crate::media_manipulation::media_utils::SECONDS_TIME_BASE;
use crate::media_manipulation::transcoding::audio::{AudioTranscoder, AudioTranscoderParams};
use crate::media_manipulation::transcoding::video::{VideoTranscoder, VideoTranscoderParams};
use crate::media_manipulation::media_utils::av_error;
use crate::media_manipulation::media_utils::in_memory_muxer::InMemoryMuxer;

mod audio;
mod video;

const PADDING_DELTA: i64 = 1;

pub fn calculate_output_width(source_width: u32, source_height: u32, target_height: u32) -> u32 {
	source_width * target_height / source_height / 2 * 2
}

pub struct TranscodingOptions<'a, B: BackendFactory> {
	pub backend_factory: &'a B,
	pub media_path: PathBuf,
	pub time_range: Range<i64>,
	pub target_video_height: u32,
	// pub target_video_framerate: u32,
	pub video_bitrate: usize,
	pub audio_bitrate: usize,
}

pub fn transcode_segment(opts: TranscodingOptions<'_, impl BackendFactory>) -> anyhow::Result<Bytes> {
	let mut demuxer = format::input(&opts.media_path).context("Opening video file")?;
	let mut muxer = InMemoryMuxer::new("mpegts").context("Opening output")?;
	
	let mut video_stream_index = usize::MAX;
	let mut audio_stream_index = usize::MAX;
	
	let mut video_transcoder = None;
	let mut audio_transcoder = None;
	
	if let Some(video_stream) = demuxer.streams().best(media::Type::Video) {
		let video_backend = opts.backend_factory.create_video_backend().context("Creating video backend")?;
		
		let params = VideoTranscoderParams {
			in_stream: &video_stream,
			muxer: &mut muxer,
			backend: video_backend,
			output_codec: Id::H264,
			target_height: opts.target_video_height,
			bit_rate: opts.video_bitrate,
			encoder_options: Dictionary::new(),
		};
		
		video_stream_index = video_stream.index();
		video_transcoder = Some(VideoTranscoder::new(params).context("Creating video transcoder")?);
	}
	
	if let Some(audio_stream) = demuxer.streams().best(media::Type::Audio) {
		let audio_codec = encoder::find(Id::AAC).unwrap().audio().context("Getting audio codec")?;
		
		let params = AudioTranscoderParams {
			in_stream: &audio_stream,
			muxer: &mut muxer,
			encoder_codec: audio_codec,
			bit_rate: opts.audio_bitrate,
			encoder_options: Dictionary::new(),
		};
		
		audio_stream_index = audio_stream.index();
		audio_transcoder = Some(AudioTranscoder::new(params).context("Creating audio transcoder")?);
	}
	
	if video_transcoder.is_none() && audio_transcoder.is_none() {
		return Err(anyhow!("Media has neither audio nor video"));
	}
	
	let mut time_bounds = opts.time_range;
	
	if time_bounds.start > 0 {
		let pos = (time_bounds.start - PADDING_DELTA).rescale(SECONDS_TIME_BASE, rescale::TIME_BASE);
		
		demuxer.seek(pos, ..pos).context("Seeking")?;
	} else {
		time_bounds.start = i64::MIN;
	}
	
	let end_time = time_bounds.end + PADDING_DELTA;
	
	for (stream, packet) in demuxer.packets() {
		if stream.index() == video_stream_index || stream.index() == audio_stream_index {
			if packet.pts().unwrap() > end_time.rescale(SECONDS_TIME_BASE, stream.time_base()) {
				break;
			}
		}
		
		if stream.index() == video_stream_index {
			if let Some(ref mut video_transcoder) = video_transcoder {
				video_transcoder.receive_input_packet(&stream, packet, time_bounds.clone())
					.context("Processing video packet")?;
			}
		} else if stream.index() == audio_stream_index {
			if let Some(ref mut audio_transcoder) = audio_transcoder {
				audio_transcoder.receive_input_packet(&stream, packet, time_bounds.clone())
					.context("Processing audio packet")?;
			}
		}
	}
	
	if let Some(ref mut video_transcoder) = video_transcoder {
		video_transcoder.send_eof(time_bounds.clone()).context("Flushing video")?;
	}
	
	if let Some(ref mut audio_transcoder) = audio_transcoder {
		audio_transcoder.send_eof(time_bounds.clone()).context("Flushing audio")?;
	}
	
	// Now mux
	
	if let Some(ref mut video_transcoder) = video_transcoder {
		video_transcoder.add_output_stream(&mut muxer).context("Adding video output stream")?;
	}
	
	unsafe {
		let mux = &mut *muxer.as_mut_ptr();
		
		if !(*mux.oformat).priv_class.is_null() && !mux.priv_data.is_null() {
			let key = CString::new("mpegts_flags").unwrap();
			let value = CString::new("+initial_discontinuity").unwrap();
			
			av_error(av_opt_set(mux.priv_data, key.as_ptr(), value.as_ptr(), 0)).context("Setting mpegts options")?;
		}
	}
	
	muxer.set_metadata(demuxer.metadata().to_owned());
	muxer.write_header().context("Writing header")?;
	
	if let Some(ref mut video_transcoder) = video_transcoder {
		video_transcoder.write_output_packets(&mut muxer).context("Writing video packets")?;
	}
	
	if let Some(ref mut audio_transcoder) = audio_transcoder {
		audio_transcoder.write_output_packets(&mut muxer).context("Writing audio packets")?;
	}
	
	muxer.write_trailer().context("Writing trailer")?;
	
	Ok(muxer.into_output_buffer().into())
}