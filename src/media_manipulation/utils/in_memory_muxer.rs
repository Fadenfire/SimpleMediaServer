use std::ffi::CString;
use std::ops::{Deref, DerefMut};
use std::ptr::{null_mut, slice_from_raw_parts};
use std::ptr;
use cfg_if::cfg_if;

use ffmpeg_next::format;
use ffmpeg_sys_next::{av_free, av_malloc, AVERROR_EOF, AVFMT_FLAG_CUSTOM_IO, avformat_alloc_output_context2, avio_alloc_context};
use turbojpeg::libc;

use crate::media_manipulation::utils::av_error;

// Why are these different?!
cfg_if! {
	if #[cfg(target_os = "macos")] {
		type BufferPointerType = *const u8;
	} else {
		type BufferPointerType = *mut u8;
	}
}

pub struct InMemoryMuxer {
	output: format::context::Output,
	boxed_buffer: Box<Option<Vec<u8>>>,
}

impl InMemoryMuxer {
	const BUFFER_SIZE: usize = 4096;
	
	pub fn new(format: &str) -> Result<Self, ffmpeg_next::Error> {
		let mut boxed_buffer = Box::new(Some(Vec::new()));
		
		unsafe {
			let box_ptr: *mut _ = boxed_buffer.as_mut();
			let mut ctx = null_mut();
			let format = CString::new(format).unwrap();
			
			av_error(avformat_alloc_output_context2(&mut ctx, ptr::null(), format.as_ptr(), ptr::null()))?;
			
			let buffer = av_malloc(Self::BUFFER_SIZE);
			
			let avio = avio_alloc_context(
				buffer.cast(), Self::BUFFER_SIZE as _,
				1,
				box_ptr.cast(),
				None,
				Some(Self::write_packet),
				None,
			);
			
			(*ctx).pb = avio;
			(*ctx).flags |= AVFMT_FLAG_CUSTOM_IO;
			
			let output = format::context::Output::wrap(ctx);
			
			Ok(Self {
				output,
				boxed_buffer,
			})
		}
	}
	
	pub fn into_output_buffer(mut self) -> Vec<u8> {
		self.boxed_buffer.take().expect("Already taken")
	}
	
	unsafe extern "C" fn write_packet(opaque: *mut libc::c_void, buf_ptr: BufferPointerType, buf_size: libc::c_int) -> libc::c_int {
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
