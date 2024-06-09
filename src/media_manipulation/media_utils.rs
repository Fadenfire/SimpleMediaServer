use std::ffi::CString;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::{null_mut, slice_from_raw_parts};

use anyhow::Context;
use ffmpeg_sys_next::{av_free, av_malloc, AVERROR_EOF, AVFMT_FLAG_CUSTOM_IO, avformat_alloc_output_context2, avio_alloc_context};
use ffmpeg_next::{decoder, Discard, format, frame, Rational};
use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::software::scaling;
use image::flat::SampleLayout;
use image::FlatSamples;
use turbojpeg::libc;

pub const SECONDS_TIME_BASE: Rational = Rational(1, 1);
pub const MILLIS_TIME_BASE: Rational = Rational(1, 1_000);
pub const MICRO_TIME_BASE: Rational = Rational(1, 1_000_000);

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

pub struct InMemoryMuxer {
	output: format::context::Output,
	boxed_buffer: Box<Option<Vec<u8>>>,
}

impl InMemoryMuxer {
	const BUFFER_SIZE: usize = 4096;
	
	pub fn new(format: &str) -> Result<Self, ffmpeg::Error> {
		let mut boxed_buffer = Box::new(Some(Vec::new()));
		
		unsafe {
			let box_ptr: *mut _ = boxed_buffer.as_mut();
			let mut ctx = null_mut();
			let format = CString::new(format).unwrap();
			
			match avformat_alloc_output_context2(&mut ctx, ptr::null(), format.as_ptr(), ptr::null()) {
				0 => {
					let buffer = av_malloc(Self::BUFFER_SIZE);
					
					let avio = avio_alloc_context(
						buffer.cast(), Self::BUFFER_SIZE as _,
						1,
						box_ptr.cast(),
						None,
						Some(Self::write_packet),
						None
					);
					
					(*ctx).pb = avio;
					(*ctx).flags |= AVFMT_FLAG_CUSTOM_IO;
					
					let output = format::context::Output::wrap(ctx);
					
					Ok(Self {
						output,
						boxed_buffer,
					})
				}
				err => Err(ffmpeg::Error::from(err))
			}
		}
	}
	
	pub fn into_output_buffer(mut self) -> Vec<u8> {
		self.boxed_buffer.take().expect("Already taken")
	}
	
	unsafe extern "C" fn write_packet(opaque: *mut libc::c_void, buf_ptr: *const u8, buf_size: libc::c_int) -> libc::c_int {
		let output_buffer_ref: *mut Option<Vec<u8>> = opaque.cast();
		
		if let Some(ref mut output_buffer) = *output_buffer_ref {
			let buf = &*slice_from_raw_parts(buf_ptr, buf_size as usize);
			output_buffer.extend_from_slice(buf);
			
			buf_size
		} else {
			AVERROR_EOF
		}
	}
}

impl Deref for InMemoryMuxer {
	type Target = format::context::Output;
	
	fn deref(&self) -> &Self::Target {
		&self.output
	}
}

impl DerefMut for InMemoryMuxer {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.output
	}
}

impl Drop for InMemoryMuxer {
	fn drop(&mut self) {
		unsafe {
			let avio = (*self.as_mut_ptr()).pb;
			(*self.as_mut_ptr()).pb = null_mut();
			
			av_free((*avio).buffer.cast());
			av_free(avio.cast());
		}
	}
}