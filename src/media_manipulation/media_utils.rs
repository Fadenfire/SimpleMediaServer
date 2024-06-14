use std::ops::Range;

use anyhow::Context;
use ffmpeg_next::{decoder, Discard, format, frame, Rational, Rescale};
use image::flat::SampleLayout;
use image::FlatSamples;

pub const SECONDS_TIME_BASE: Rational = Rational(1, 1);
pub const MILLIS_TIME_BASE: Rational = Rational(1, 1_000);
pub const MICRO_TIME_BASE: Rational = Rational(1, 1_000_000);

pub fn scale_range(range: Range<i64>, from: Rational, to: Rational) -> Range<i64> {
	Range {
		start: range.start.rescale(from, to),
		end: range.end.rescale(from, to),
	}
}

pub fn discard_all_but_keyframes(demuxer: &mut format::context::Input, stream_index: usize) {
	for mut stream in demuxer.streams_mut() {
		let discard = if stream.index() == stream_index { Discard::NonKey } else { Discard::All };
		unsafe { (*stream.as_mut_ptr()).discard = discard.into(); }
	}
}

pub fn push_one_packet(
	demuxer: &mut format::context::Input,
	decoder: &mut decoder::Video,
	stream_index: usize,
) -> anyhow::Result<()> {
	for (stream, packet) in demuxer.packets() {
		if stream.index() == stream_index && packet.is_key() {
			decoder.send_packet(&packet).context("Decoding packet")?;
			break;
		}
	}
	
	Ok(())
}

pub fn frame_image_sample_rgb(frame: &frame::Video) -> FlatSamples<&[u8]> {
	let components = frame.format().descriptor().unwrap().nb_components();
	
	let layout = SampleLayout {
		channels: components,
		channel_stride: 1,
		width: frame.width(),
		width_stride: components as usize,
		height: frame.height(),
		height_stride: frame.stride(0),
	};
	
	let samples = FlatSamples {
		samples: frame.data(0),
		layout,
		color_hint: None,
	};
	
	samples
}