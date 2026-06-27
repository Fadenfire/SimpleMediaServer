use std::ops::Range;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use bytes::Bytes;
use ffmpeg_next::{codec, encoder, format, media, rescale, Dictionary, Rescale};

use crate::media_manipulation::backends::BackendFactory;
use crate::media_manipulation::media_utils;
use crate::media_manipulation::media_utils::in_memory_muxer::InMemoryMuxer;
use crate::media_manipulation::media_utils::scale_from_f64_secs;
use crate::media_manipulation::transcoding::audio::{AudioTranscoder, AudioTranscoderParams};
use crate::media_manipulation::transcoding::video::{VideoTranscoder, VideoTranscoderParams};

mod audio;
mod video;
pub mod subtitle;

const START_PADDING: f64 = 0.3;
const END_PADDING: f64 = 0.15;

pub fn calculate_output_width(source_width: u32, source_height: u32, target_height: u32) -> u32 {
	source_width * target_height / source_height / 2 * 2
}

pub struct TranscodingOptions<'a> {
	pub backend_factory: &'a dyn BackendFactory,
	pub media_path: PathBuf,
	pub target_video_height: u32,
	// pub target_video_framerate: u32,
	pub video_codec: codec::Id,
	pub video_bitrate: usize,
	pub audio_bitrate: usize,
}

pub fn transcode_segment(opts: TranscodingOptions, mut time_bounds: Range<f64>) -> anyhow::Result<Bytes> {
	let mut demuxer = format::input(&opts.media_path).context("Opening video file")?;
	let mut muxer = InMemoryMuxer::new("mpegts").context("Opening output")?;
	
	let mut video_stream_index = usize::MAX;
	let mut audio_stream_index = usize::MAX;
	
	let mut video_transcoder = None;
	let mut audio_transcoder = None;
	
	if let Some(video_stream) = demuxer.streams().best(media::Type::Video) {
		let video_backend = opts.backend_factory.create_video_backend()
			.context("Creating video backend")?;
		
		let params = VideoTranscoderParams {
			in_stream: &video_stream,
			muxer: &muxer,
			backend: video_backend,
			output_codec: opts.video_codec,
			target_height: opts.target_video_height,
			bit_rate: opts.video_bitrate,
			encoder_options: Dictionary::new(),
		};
		
		video_stream_index = video_stream.index();
		video_transcoder = Some(VideoTranscoder::new(params).context("Creating video transcoder")?);
	}
	
	if let Some(audio_stream) = demuxer.streams().best(media::Type::Audio) {
		let audio_codec = encoder::find(codec::Id::AAC).unwrap().audio()
			.context("Getting audio codec")?;
		
		let params = AudioTranscoderParams {
			in_stream: &audio_stream,
			muxer: &muxer,
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
	
	media_utils::seek_to_bounds_beginning(&mut demuxer, &mut time_bounds, START_PADDING).context("Seeking")?;
	
	let container_start_time = media_utils::demuxer_start_time(&demuxer);
	let end_dts = scale_from_f64_secs(time_bounds.end + END_PADDING, rescale::TIME_BASE) + container_start_time;
	
	let mut has_more_video = video_transcoder.is_some();
	let mut has_more_audio = audio_transcoder.is_some();
	
	for (stream, packet) in demuxer.packets() {
		if [video_stream_index, audio_stream_index].contains(&stream.index()) {
			let dts = packet.dts()
				.ok_or_else(|| anyhow!("Video/audio packet had no DTS"))?;
			
			if dts > end_dts.rescale(rescale::TIME_BASE, stream.time_base()) {
				match stream.index() {
					i if i == video_stream_index => has_more_video = false,
					i if i == audio_stream_index => has_more_audio = false,
					_ => {}
				}
			}
		}
		
		if !has_more_video && !has_more_audio {
			break;
		}
		
		if has_more_video && stream.index() == video_stream_index {
			if let Some(ref mut video_transcoder) = video_transcoder {
				video_transcoder.receive_input_packet(&stream, packet, time_bounds.clone())
					.context("Processing video packet")?;
			}
		}
		else if has_more_audio && stream.index() == audio_stream_index {
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
	
	if let Some(ref mut audio_transcoder) = audio_transcoder {
		audio_transcoder.add_output_stream(&mut muxer).context("Adding video output stream")?;
	}
	
	let mut mux_options = Dictionary::new();
	mux_options.set("mpegts_flags", "+initial_discontinuity");
	
	muxer.write_header_with(mux_options).context("Writing header")?;
	
	if let Some(ref mut video_transcoder) = video_transcoder {
		video_transcoder.write_output_packets(&mut muxer).context("Writing video packets")?;
	}
	
	if let Some(ref mut audio_transcoder) = audio_transcoder {
		audio_transcoder.write_output_packets(&mut muxer).context("Writing audio packets")?;
	}
	
	muxer.write_trailer().context("Writing trailer")?;
	
	Ok(muxer.into_output_buffer().into())
}