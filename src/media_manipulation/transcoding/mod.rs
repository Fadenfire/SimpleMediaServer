use std::ffi::CString;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use bytes::Bytes;
use ffmpeg_sys_the_third::av_opt_set;
use ffmpeg_the_third::{Dictionary, encoder, format, media, Rescale, rescale};
use ffmpeg_the_third::codec::Id;
use crate::media_manipulation::backends::video_toolbox::VideoToolboxVideoBackend;

use crate::media_manipulation::media_utils::{InMemoryMuxer, SECONDS_TIME_BASE};
use crate::media_manipulation::transcoding::audio::{AudioTranscoder, AudioTranscoderParams};
use crate::media_manipulation::transcoding::video::{VideoTranscoder, VideoTranscoderParams};

mod audio;
mod video;

const PADDING_DELTA: i64 = 1;

pub fn transcode_segment(media_path: PathBuf, segment_index: usize, segment_size: i64) -> anyhow::Result<Bytes> {
	let mut demuxer = format::input(&media_path).context("Opening video file")?;
	let mut muxer = InMemoryMuxer::new("mpegts").context("Opening output")?;
	
	let mut video_backend = VideoToolboxVideoBackend::new();
	
	let mut video_stream_index = usize::MAX;
	let mut audio_stream_index = usize::MAX;
	
	let mut video_transcoder = None;
	let mut audio_transcoder = None;
	
	if let Some(video_stream) = demuxer.streams().best(media::Type::Video) {
		let params = VideoTranscoderParams {
			in_stream: &video_stream,
			muxer: &mut muxer,
			backend: &mut video_backend,
			output_codec: Id::H264,
			bit_rate: 12_000_000,
			encoder_options: Dictionary::new()
		};
		
		video_stream_index = video_stream.index();
		video_transcoder = Some(VideoTranscoder::new(params).context("Creating video transcoder")?);
	}
	
	if let Some(audio_stream) = demuxer.streams().best(media::Type::Audio) {
		let audio_codec = encoder::find(Id::AAC).unwrap().audio().context("Getting audio codec")?;
		
		let params = AudioTranscoderParams {
			in_stream: &audio_stream,
			muxer: &mut muxer,
			decoder_codec: None,
			encoder_codec: audio_codec,
			bit_rate: 160_000,
			encoder_options: Dictionary::new()
		};
		
		audio_stream_index = audio_stream.index();
		audio_transcoder = Some(AudioTranscoder::new(params).context("Creating audio transcoder")?);
	}
	
	if video_transcoder.is_none() && audio_transcoder.is_none() {
		return Err(anyhow!("Media has neither audio nor video"));
	}
	
	unsafe {
		let mux = &mut *muxer.as_mut_ptr();
		
		if !(*mux.oformat).priv_class.is_null() && !mux.priv_data.is_null() {
			let key = CString::new("mpegts_flags").unwrap();
			let value = CString::new("+initial_discontinuity").unwrap();
			
			av_opt_set(mux.priv_data, key.as_ptr(), value.as_ptr(), 0);
		}
	}
	
	muxer.set_metadata(demuxer.metadata().to_owned());
	muxer.write_header().context("Writing header")?;
	
	let mut time_bounds = (segment_index as i64 * segment_size)..((segment_index + 1) as i64 * segment_size);
	
	if time_bounds.start > 0 {
		let pos = (time_bounds.start - PADDING_DELTA).rescale(SECONDS_TIME_BASE, rescale::TIME_BASE);
		
		demuxer.seek(pos, ..pos).context("Seeking")?;
	} else {
		time_bounds.start = -10000;
	}
	
	let end_time = time_bounds.end + PADDING_DELTA;
	
	for result in demuxer.packets() {
		let (stream, packet) = result?;
		
		if stream.index() == video_stream_index || stream.index() == audio_stream_index {
			if packet.pts().unwrap() > end_time.rescale(SECONDS_TIME_BASE, stream.time_base()) {
				break;
			}
		}
		
		if stream.index() == video_stream_index {
			if let Some(ref mut video_transcoder) = video_transcoder {
				video_transcoder.receive_input_packet(&stream, packet, &mut muxer, time_bounds.clone()).context("Processing video packet")?;
			}
		} else if stream.index() == audio_stream_index {
			if let Some(ref mut audio_transcoder) = audio_transcoder {
				audio_transcoder.receive_input_packet(packet, &mut muxer, time_bounds.clone()).context("Processing audio packet")?;
			}
		}
	}
	
	if let Some(ref mut video_transcoder) = video_transcoder {
		video_transcoder.send_eof(&mut muxer, time_bounds.clone()).context("Flushing video")?;
	}
	
	if let Some(ref mut audio_transcoder) = audio_transcoder {
		audio_transcoder.send_eof(&mut muxer, time_bounds.clone()).context("Flushing audio")?;
	}
	
	muxer.write_trailer().context("Writing trailer")?;
	
	Ok(muxer.into_output_buffer().into())
}