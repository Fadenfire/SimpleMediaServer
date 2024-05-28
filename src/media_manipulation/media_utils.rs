use anyhow::Context;
use ffmpeg_the_third::{decoder, Discard, format, frame, Rational};
use ffmpeg_the_third::format::Pixel;
use ffmpeg_the_third::software::scaling;
use image::flat::SampleLayout;
use image::FlatSamples;

pub const SECONDS_TIME_BASE: Rational = Rational(1, 1);
pub const MILLIS_TIME_BASE: Rational = Rational(1, 1_000);
pub const MICRO_TIME_BASE: Rational = Rational(1, 1_000_000);

pub fn set_decoder_time_base(decoder: &mut decoder::Video, time_base: Rational) {
	unsafe {
		(*decoder.as_mut_ptr()).pkt_timebase = time_base.into();
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
	for result in demuxer.packets() {
		let (stream, packet) = result?;
		
		if stream.index() == stream_index && packet.is_key() {
			decoder.send_packet(&packet).context("Decoding packet")?;
			break;
		}
	}
	
	Ok(())
}

pub fn scale_frame_rgb(
	cache: &mut Option<scaling::Context>,
	in_frame: &frame::Video,
	out_width: u32,
	out_height: u32,
) -> anyhow::Result<frame::Video> {
	let context = match cache {
		Some(ctx) => {
			ctx.cached(
				in_frame.format(), in_frame.width(), in_frame.height(),
				Pixel::RGB24, out_width, out_height,
				scaling::Flags::BICUBIC,
			);
			
			ctx
		}
		None => {
			*cache = Some(scaling::Context::get(
				in_frame.format(), in_frame.width(), in_frame.height(),
				Pixel::RGB24, out_width, out_height,
				scaling::Flags::BICUBIC,
			).context("Creating image converter")?);
			
			cache.as_mut().unwrap()
		}
	};
	
	let mut out_frame = frame::Video::empty();
	context.run(in_frame, &mut out_frame).context("Converting frame")?;
	
	Ok(out_frame)
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