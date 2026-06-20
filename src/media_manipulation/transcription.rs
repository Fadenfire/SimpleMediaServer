use std::fmt::Write;
use crate::media_manipulation::media_utils::{scale_f64_secs, MILLIS_TIME_BASE, SECONDS_TIME_BASE};
use anyhow::{anyhow, Context};
use ffmpeg_next::format::{sample, Sample};
use ffmpeg_next::software::resampling;
use ffmpeg_next::{codec, decoder, format, frame, media, rescale, ChannelLayout, Packet, Rescale, Stream};
use parakeet_rs::{ParakeetTDT, TimedToken, TimestampMode, Transcriber};
use std::ops::Range;
use std::path::PathBuf;
use crate::media_manipulation::media_utils;

const SAMPLE_RATE: u32 = 16_000;

const OVERLAP_SECONDS: f64 = 10.0;

pub fn transcribe(
	media_path: PathBuf,
	mut parakeet: ParakeetTDT,
	mut time_bounds: Range<f64>,
) -> anyhow::Result<String> {
	let mut demuxer = format::input(&media_path).context("Opening video file")?;
	
	let audio_stream = demuxer.streams().best(media::Type::Audio)
		.ok_or_else(|| anyhow::anyhow!("Media file has no audio"))?;
	
	let audio_stream_index = audio_stream.index();
	
	let mut sample_collector = SampleCollector::new(audio_stream)?;
	
	let container_start_time = media_utils::demuxer_start_time(&demuxer);
	
	if time_bounds.start > 0.0 {
		let start_time_secs = time_bounds.start + OVERLAP_SECONDS;
		let seek_pos = scale_f64_secs(start_time_secs, rescale::TIME_BASE) + container_start_time;
		
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
	
	
	
	let samples = sample_collector.samples;
	
	const CHANNEL_COUNT: u16 = 1;
	
	let result = parakeet.transcribe_samples(
		samples,
		SAMPLE_RATE,
		CHANNEL_COUNT,
		Some(TimestampMode::Words)
	).context("Transcribing")?;
	
	println!("{}", build_webvtt(&result.tokens, 0.0..60.0));
	
	Ok(())
}

#[derive(Debug, Clone)]
pub struct VTTCue {
	text: String,
	start_time: f64,
	end_time: f64,
}

const MAX_SUB_LENGTH: usize = 20;
const MAX_SILENCE_GAP: f32 = 2.0;

fn build_webvtt(words: &[TimedToken], time_range: Range<f32>) -> String {
	let mut webvtt = String::new();
	webvtt.push_str("WEBVTT\n");
	
	let mut line = String::new();
	let mut line_word_count = 0;
	let mut line_start = 0f32;
	let mut line_end = 0f32;
	
	for word in words {
		if !time_range.contains(&word.start) {
			continue;
		}

		if !line.is_empty() {
			if
				line.ends_with(|c| matches!(c, '.' | '!' | '?')) ||
				line_word_count >= MAX_SUB_LENGTH ||
				word.start - line_end >= MAX_SILENCE_GAP
			{
				webvtt.push('\n');
				write_cue(&mut webvtt, &line, line_start, line_end);
				
				line.clear();
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
		webvtt.push('\n');
		write_cue(&mut webvtt, &line, line_start, line_end);
	}
	
	webvtt
}

fn write_cue(text: &mut String, line: &str, start: f32, end: f32) {
	write_timestamp(text, start);
	text.push_str(" --> ");
	write_timestamp(text, end);
	text.push('\n');
	
	text.push_str(&line);
	text.push('\n');
}

fn write_timestamp(text: &mut String, time: f32) {
	let minutes = (time / 60.0).floor() as u32;
	let seconds = (time % 60.0).floor() as u32;
	let milliseconds = (time % 1.0 * 1000.0).round() as u32;
	
	text.push_str(&format!("{:02}:{:02}.{:03}", minutes, seconds, milliseconds));
}

struct SampleCollector {
	decoder: decoder::Audio,
	resampler: resampling::Context,
	
	samples: Vec<f32>,
}

impl SampleCollector {
	pub fn new(audio_stream: Stream) -> anyhow::Result<Self> {
		let mut decoder = codec::context::Context::from_parameters(audio_stream.parameters())?
			.decoder()
			.audio()?;
		
		decoder.set_packet_time_base(audio_stream.time_base());
		
		let resampler = ffmpeg_next::software::resampler(
			(decoder.format(), decoder.channel_layout(), decoder.rate()),
			(Sample::F32(sample::Type::Packed), ChannelLayout::MONO, SAMPLE_RATE)
		)?;
		
		Ok(Self {
			decoder,
			resampler,
			
			samples: Vec::new(),
		})
	}
	
	pub fn receive_input_packet(&mut self, in_packet: Packet) -> anyhow::Result<()> {
		self.decoder.send_packet(&in_packet).context("Sending packet")?;
		self.decode_frames()
	}
	
	pub fn send_eof(&mut self) -> anyhow::Result<()> {
		self.decoder.send_eof()?;
		self.decode_frames()?;
		
		Ok(())
	}
	
	fn decode_frames(&mut self) -> anyhow::Result<()> {
		let mut in_frame = frame::Audio::empty();
		
		while self.decoder.receive_frame(&mut in_frame).is_ok() {
			let mut out_frame = frame::Audio::empty();
			self.resampler.run(&in_frame, &mut out_frame)?;
			
			self.samples.extend(out_frame.plane(0));
		}
		
		Ok(())
	}
}