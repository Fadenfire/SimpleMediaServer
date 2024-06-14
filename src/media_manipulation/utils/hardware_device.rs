use std::ptr::{null, null_mut};

use ffmpeg_sys_next::{av_buffer_ref, av_buffer_unref, av_hwdevice_ctx_create, AVBufferRef, AVHWDeviceType};

use crate::media_manipulation::utils::av_error;

pub struct HardwareDeviceContext {
	ptr: *mut AVBufferRef,
}

impl HardwareDeviceContext {
	pub fn create(device_type: AVHWDeviceType) -> Result<Self, ffmpeg_next::Error> {
		unsafe {
			let mut ptr = null_mut();
			av_error(av_hwdevice_ctx_create(&mut ptr, device_type, null(), null_mut(), 0))?;
			
			Ok(Self {
				ptr,
			})
		}
	}
	
	pub fn add_ref(&self) -> Result<*mut AVBufferRef, ffmpeg_next::Error> {
		let new_ptr = unsafe { av_buffer_ref(self.ptr) };
		
		if new_ptr.is_null() {
			Err(ffmpeg_next::Error::Unknown)
		} else {
			Ok(new_ptr)
		}
	}
}

impl Drop for HardwareDeviceContext {
	fn drop(&mut self) {
		unsafe {
			av_buffer_unref(&mut self.ptr);
		}
	}
}