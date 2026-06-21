use std::ffi::{c_int, c_void};
use std::num::NonZeroUsize;
use std::ops::Range;

use anyhow::Context;
use ffmpeg_next::{decoder, Discard, format, frame, Rational, Rescale, Stream, encoder, codec, StreamMut};
use ffmpeg_next::packet::Mut;
use ffmpeg_sys_next::{AVERROR, ENOMEM};
use image::flat::SampleLayout;
use image::FlatSamples;

pub mod in_memory_muxer;
pub mod hardware_device;
pub mod frame_scaler;
pub mod resource_pool;

pub const SECONDS_TIME_BASE: Rational = Rational(1, 1);
pub const MILLIS_TIME_BASE: Rational = Rational(1, 1_000);
pub const MICRO_TIME_BASE: Rational = Rational(1, 1_000_000);

pub fn scale_from_f64_secs(timestamp: f64, new_scale: Rational) -> i64 {
	let micro_seconds = (timestamp * 1_000_000.0).round() as i64;
	
	micro_seconds.rescale(MICRO_TIME_BASE, new_scale)
}

pub fn scale_to_f64_secs(timestamp: i64, old_scale: Rational) -> f64 {
	let micro_seconds = timestamp.rescale(old_scale, MICRO_TIME_BASE);
	
	micro_seconds as f64 / 1_000_000.0
}

pub fn scale_range(range: Range<i64>, from: Rational, to: Rational) -> Range<i64> {
	Range {
		start: range.start.rescale(from, to),
		end: range.end.rescale(from, to),
	}
}

pub fn demuxer_start_time(demuxer: &format::context::Input) -> i64 {
	unsafe {
		(*demuxer.as_ptr()).start_time
	}
}

pub fn discard_all_but_one(demuxer: &mut format::context::Input, stream_index: usize, stream_discard: Discard) {
	discard_streams(demuxer, |stream| {
		if stream.index() == stream_index { stream_discard } else { Discard::All }
	});
}

pub fn discard_streams(
	demuxer: &mut format::context::Input,
	mut stream_discard: impl FnMut(&Stream) -> Discard
) {
	for mut stream in demuxer.streams_mut() {
		let discard = stream_discard(&stream);
		
		unsafe { (*stream.as_mut_ptr()).discard = discard.into(); }
	}
}

pub fn push_one_packet(
	demuxer: &mut format::context::Input,
	decoder: &mut decoder::Video,
	stream_index: usize,
	opaque_value: Option<NonZeroUsize>,
) -> anyhow::Result<()> {
	for (stream, mut packet) in demuxer.packets() {
		if stream.index() == stream_index && packet.is_key() {
			if let Some(opaque) = opaque_value {
				unsafe { (*packet.as_mut_ptr()).opaque = opaque.get() as *mut c_void; }
			}
			
			decoder.send_packet(&packet).context("Decoding packet")?;
			break;
		}
	}
	
	Ok(())
}

pub fn get_frame_opaque(frame: &ffmpeg_next::Frame) -> Option<NonZeroUsize> {
	let opaque = unsafe { (*frame.as_ptr()).opaque };
	
	if opaque.is_null() {
		None
	} else {
		NonZeroUsize::new(opaque as usize)
	}
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

pub fn add_output_stream(
	muxer: &'_ mut format::context::Output,
	params: impl Into<codec::Parameters>
) -> anyhow::Result<StreamMut<'_>> {
	let mut out_stream = muxer.add_stream(encoder::find(codec::Id::None))?;
	out_stream.set_parameters(params);
	
	unsafe { (*(*out_stream.as_mut_ptr()).codecpar).codec_tag = 0; }
	
	Ok(out_stream)
}

pub fn av_error(code: c_int) -> Result<c_int, ffmpeg_next::Error> {
	match code {
		0.. => Ok(code),
		_ => Err(ffmpeg_next::Error::from(code)),
	}
}

pub fn check_alloc<T>(ptr: *mut T) -> Result<*mut T, ffmpeg_next::Error> {
	if ptr.is_null() {
		Err(ffmpeg_next::Error::from(AVERROR(ENOMEM)))
	} else {
		Ok(ptr)
	}
}
