use crate::media_manipulation::media_utils;
use crate::media_manipulation::media_utils::{scale_from_f64_secs, scale_to_f64_secs};
use crate::media_manipulation::transcription::decoding::SampleCollector;
use anyhow::{anyhow, Context};
use ffmpeg_next::{format, media, rescale, Rescale};
use parakeet_rs::{ParakeetTDT, TimedToken, TimestampMode, Transcriber};
use std::ops::Range;
use std::path::PathBuf;
use tracing::info;
use web_vtt::VTTCue;

mod web_vtt;
mod decoding;

const SAMPLE_RATE: u32 = 16_000;

pub fn transcribe(
	media_path: PathBuf,
	parakeet: &mut ParakeetTDT,
	mut time_bounds: Range<f64>,
	overlap: f64,
) -> anyhow::Result<String> {
	let mut demuxer = format::input(&media_path).context("Opening video file")?;
	
	let audio_stream = demuxer.streams().best(media::Type::Audio)
		.ok_or_else(|| anyhow::anyhow!("Media file has no audio"))?;
	
	let audio_stream_index = audio_stream.index();
	
	let mut sample_collector = SampleCollector::new(audio_stream)?;
	
	let container_start_time = media_utils::demuxer_start_time(&demuxer);
	let mut decoding_start = None;
	
	if time_bounds.start > 0.0 {
		let seek_pos_secs = (time_bounds.start - overlap).max(0.0);
		let seek_pos = scale_from_f64_secs(seek_pos_secs, rescale::TIME_BASE) + container_start_time;
		
		demuxer.seek(seek_pos, ..seek_pos).context("Seeking")?;
	} else {
		time_bounds.start = f64::MIN;
	}
	
	let last_dts = scale_from_f64_secs(time_bounds.end + overlap, rescale::TIME_BASE) + container_start_time;
	
	for (stream, packet) in demuxer.packets() {
		if stream.index() == audio_stream_index {
			let dts = packet.dts()
				.ok_or_else(|| anyhow!("Audio packet had no DTS"))?;
			
			if decoding_start.is_none() {
				let f_dts = scale_to_f64_secs(dts, stream.time_base())
					- scale_to_f64_secs(container_start_time, rescale::TIME_BASE);
				
				decoding_start = Some(f_dts);
			}
	
			if dts > last_dts.rescale(rescale::TIME_BASE, stream.time_base()) {
				break;
			}
			
			sample_collector.receive_input_packet(packet)?;
		}
	}
	
	sample_collector.send_eof()?;
	
	let samples = sample_collector.into_samples();
	let decoding_start = decoding_start.unwrap_or(time_bounds.start);
	
	let result = parakeet.transcribe_samples(
		samples,
		SAMPLE_RATE,
		1, // Channel count
		Some(TimestampMode::Tokens)
	).context("Transcribing")?;
	
	info!("Transcribed: {:?}", &result.tokens);
	
	let cues = build_transcript(
		result.tokens.into_iter()
			.map(|token| TimedToken {
				start: token.start + decoding_start as f32,
				end: token.end + decoding_start as f32,
				..token
			})
			.filter(|token| time_bounds.contains(&(token.start as f64)))
	);
	
	let web_vtt = web_vtt::cues_to_web_vtt(&cues);
	
	Ok(web_vtt)
}

const MAX_SUB_LENGTH: usize = 200;
const MAX_SILENCE_GAP: f32 = 2.0;

fn build_transcript(words: impl IntoIterator<Item = TimedToken>) -> Vec<VTTCue> {
	let mut cues = Vec::new();
	
	let mut line = String::new();
	let mut line_start = 0f32;
	let mut line_end = 0f32;
	
	for word in words {
		if !line.is_empty() {
			if
				line.ends_with(|c| matches!(c, '.' | '!' | '?')) ||
				(line.len() >= MAX_SUB_LENGTH && line.ends_with(char::is_whitespace)) ||
				(word.start - line_end) >= MAX_SILENCE_GAP
			{
				let final_line = line
					.trim_start_matches(|c: char| {
						c.is_whitespace() || matches!(c, '.' | ',' | '!' | '?' | ';' | ':' | ')')
					})
					.trim()
					.to_string();
				
				cues.push(VTTCue {
					text: final_line,
					start_time: line_start,
					end_time: line_end,
				});
				
				line.clear();
			}
		}
		
		if line.is_empty() {
			line_start = word.start;
		}
		
		line_end = word.start + 0.01;
		
		line.push_str(&word.text);
	}
	
	if !line.is_empty() {
		cues.push(VTTCue {
			text: line,
			start_time: line_start,
			end_time: line_end,
		});
	}
	
	cues
}
