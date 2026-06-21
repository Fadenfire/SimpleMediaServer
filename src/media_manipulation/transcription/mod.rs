use crate::media_manipulation::media_utils;
use crate::media_manipulation::media_utils::{scale_from_f64_secs, scale_to_f64_secs};
use crate::media_manipulation::transcription::decoding::SampleCollector;
use anyhow::{anyhow, Context};
use ffmpeg_next::{format, media, rescale, Rescale};
use parakeet_rs::{ParakeetTDT, TimedToken, TimestampMode, Transcriber};
use std::ops::Range;
use std::path::PathBuf;
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
	
	if time_bounds.start > 0.0 {
		let seek_pos_secs = (time_bounds.start - overlap).max(0.0);
		let seek_pos = scale_from_f64_secs(seek_pos_secs, rescale::TIME_BASE) + container_start_time;
		
		demuxer.seek(seek_pos, ..seek_pos).context("Seeking")?;
	} else {
		time_bounds.start = f64::MIN;
	}
	
	let end_dts = scale_from_f64_secs(time_bounds.end + overlap, rescale::TIME_BASE) + container_start_time;
	
	for (stream, packet) in demuxer.packets() {
		if stream.index() == audio_stream_index {
			let dts = packet.dts()
				.ok_or_else(|| anyhow!("Audio packet had no DTS"))?;
			
			if dts > end_dts.rescale(rescale::TIME_BASE, stream.time_base()) {
				break;
			}
			
			sample_collector.receive_input_packet(packet)?;
		}
	}
	
	sample_collector.send_eof()?;
	
	let first_pts = sample_collector.first_pts()
		.unwrap_or(0.0)
		- scale_to_f64_secs(container_start_time, rescale::TIME_BASE);
	
	let samples = sample_collector.into_samples();
	
	let result = parakeet.transcribe_samples(
		samples,
		SAMPLE_RATE,
		1, // Channel count
		Some(TimestampMode::Tokens)
	).context("Transcribing")?;
	
	let cues = build_transcript(
		result.tokens.into_iter()
			.map(|token| TimedToken {
				start: token.start + first_pts as f32,
				end: token.end + first_pts as f32,
				..token
			})
			.filter(|token| time_bounds.contains(&(token.start as f64)))
	);
	
	let web_vtt = web_vtt::cues_to_web_vtt(&cues);
	
	Ok(web_vtt)
}

const MAX_SUB_LENGTH: usize = 45;
const MAX_SILENCE_GAP: f32 = 1.2;

fn build_transcript(tokens: impl IntoIterator<Item = TimedToken>) -> Vec<VTTCue> {
	let mut cues = Vec::new();
	
	let mut line = String::new();
	let mut line_start = 0f32;
	let mut line_end = 0f32;
	
	let mut append_cue = |
		text: &str,
		start_time: f32,
		end_time: f32,
	| {
		let final_text = text
			.trim_start_matches(|c: char| {
				c.is_whitespace() || matches!(c, '.' | ',')
			})
			.trim()
			.to_string();
		
		if !final_text.is_empty() {
			cues.push(VTTCue {
				text: final_text,
				start_time,
				end_time,
			});
		}
	};
	
	for token in tokens {
		if !line.is_empty() {
			if
				line.ends_with(|c| matches!(c, '.' | '!' | '?')) ||
				(
					line.len() >= MAX_SUB_LENGTH &&
					(line.ends_with(char::is_whitespace) || token.text.starts_with(char::is_whitespace))
				) ||
				(token.start - line_end) >= MAX_SILENCE_GAP
			{
				append_cue(&line, line_start, line_end);
				
				line.clear();
			}
		}
		
		if line.is_empty() {
			line_start = token.start;
		}
		
		line_end = token.end.min(token.start + 0.2);
		
		line.push_str(&token.text);
	}
	
	if !line.is_empty() {
		append_cue(&line, line_start, line_end);
	}
	
	cues
}
