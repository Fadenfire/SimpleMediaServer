use crate::media_manipulation::media_utils;
use crate::media_manipulation::media_utils::scale_f64_secs;
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

const OVERLAP_SECONDS: f64 = 10.0;

pub fn transcribe(
	media_path: PathBuf,
	parakeet: &mut ParakeetTDT,
	mut time_bounds: Range<f64>,
) -> anyhow::Result<String> {
	let mut demuxer = format::input(&media_path).context("Opening video file")?;
	
	let audio_stream = demuxer.streams().best(media::Type::Audio)
		.ok_or_else(|| anyhow::anyhow!("Media file has no audio"))?;
	
	let audio_stream_index = audio_stream.index();
	
	let mut sample_collector = SampleCollector::new(audio_stream)?;
	
	let container_start_time = media_utils::demuxer_start_time(&demuxer);
	let mut decoding_start = time_bounds.start;
	
	if time_bounds.start > 0.0 {
		decoding_start = (decoding_start - OVERLAP_SECONDS).max(0.0);
		let seek_pos = scale_f64_secs(decoding_start, rescale::TIME_BASE) + container_start_time;
		
		demuxer.seek(seek_pos, ..seek_pos).context("Seeking")?;
	} else {
		time_bounds.start = f64::MIN;
	}
	
	let last_dts = scale_f64_secs(time_bounds.end + OVERLAP_SECONDS, rescale::TIME_BASE) + container_start_time;
	
	for (stream, packet) in demuxer.packets() {
		if stream.index() == audio_stream_index {
			let dts = packet.dts()
				.ok_or_else(|| anyhow!("Audio packet had no DTS"))?;
	
			if dts > last_dts.rescale(rescale::TIME_BASE, stream.time_base()) {
				break;
			}
			
			sample_collector.receive_input_packet(packet)?;
		}
	}
	
	sample_collector.send_eof()?;
	
	let samples = sample_collector.into_samples();
	
	let result = parakeet.transcribe_samples(
		samples,
		SAMPLE_RATE,
		1, // Channel count
		Some(TimestampMode::Words)
	).context("Transcribing")?;
	
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

const MAX_SUB_LENGTH: usize = 20;
const MAX_SILENCE_GAP: f32 = 2.0;

fn build_transcript(words: impl IntoIterator<Item = TimedToken>) -> Vec<VTTCue> {
	let mut cues = Vec::new();
	
	let mut line = String::new();
	let mut line_word_count = 0;
	let mut line_start = 0f32;
	let mut line_end = 0f32;
	
	for word in words {
		if !line.is_empty() {
			if
				line.ends_with(|c| matches!(c, '.' | '!' | '?')) ||
				line_word_count >= MAX_SUB_LENGTH ||
				word.start - line_end >= MAX_SILENCE_GAP
			{
				cues.push(VTTCue {
					text: line,
					start_time: line_start,
					end_time: line_end,
				});
				
				line = String::new();
				line_word_count = 0;
			}
		}
		
		let is_punct = matches!(word.text.as_str(), "." | "," | "!" | "?" | ";" | ":" | ")");
		
		if line.is_empty() {
			line_start = word.start;
		} else if !is_punct {
			line.push(' ');
		}
		
		line_end = word.end;
		line_word_count += 1;
		
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
