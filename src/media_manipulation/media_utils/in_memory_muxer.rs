use std::ffi::CString;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::null_mut;
use anyhow::anyhow;
use ffmpeg_next::format;
use ffmpeg_sys_next::{av_free, av_write_frame, avformat_alloc_output_context2, avio_close_dyn_buf, avio_open_dyn_buf};

use crate::media_manipulation::media_utils::av_error;

pub struct InMemoryMuxer {
	output: format::context::Output,
}

impl InMemoryMuxer {
	pub fn new(format: &str) -> Result<Self, ffmpeg_next::Error> {
		unsafe {
			let mut ctx = null_mut();
			let format = CString::new(format).unwrap();
			
			av_error(avformat_alloc_output_context2(&mut ctx, ptr::null(), format.as_ptr(), ptr::null()))?;
			av_error(avio_open_dyn_buf(&mut (*ctx).pb))?;
			
			let output = format::context::Output::wrap(ctx);
			
			Ok(Self { output })
		}
	}
	
	pub fn flush(&mut self) -> Result<(), ffmpeg_next::Error> {
		unsafe {
			av_error(av_write_frame(self.output.as_mut_ptr(), null_mut()))?;
		}
		
		Ok(())
	}
	
	pub fn drain_output_buffer(&mut self) -> anyhow::Result<Vec<u8>> {
		self.flush()?;
		
		unsafe {
			let ctx = self.output.as_mut_ptr();
			
			let mut av_buffer = null_mut();
			let size = avio_close_dyn_buf((*ctx).pb, &mut av_buffer);
			
			if av_buffer.is_null() {
				return Err(anyhow!("avio_close_dyn_buf returned null buffer"));
			}
			
			let av_buffer = scopeguard::guard(av_buffer, |ptr| av_free(ptr.cast()));
			
			(*ctx).pb = null_mut();
			av_error(avio_open_dyn_buf(&mut (*ctx).pb))?;
			
			let buffer = std::slice::from_raw_parts(*av_buffer, size as usize).to_vec();
			
			Ok(buffer)
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
			let ctx = self.output.as_mut_ptr();
			
			let avio = (*ctx).pb;
			(*ctx).pb = null_mut();
			
			let mut av_buffer = null_mut();
			avio_close_dyn_buf(avio, &mut av_buffer);
			
			av_free(av_buffer.cast());
		}
	}
}
