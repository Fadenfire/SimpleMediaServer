use ffmpeg_sys_next::{av_buffer_ref, av_buffer_unref, av_hwdevice_ctx_create, AVBufferRef, AVHWDeviceType};
use std::ptr::{null, null_mut};

use crate::media_manipulation::media_utils::resource_pool::{BorrowedResource, ResourcePool};
use crate::media_manipulation::media_utils::{av_error, check_alloc};

pub type DevicePool = ResourcePool<HardwareDeviceContext>;
pub type BorrowedDevice = BorrowedResource<HardwareDeviceContext>;

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
		unsafe { check_alloc(av_buffer_ref(self.ptr)) }
	}
}

unsafe impl Send for HardwareDeviceContext {}

impl Clone for HardwareDeviceContext {
	fn clone(&self) -> Self {
		Self {
			ptr: self.add_ref().expect("Failed to clone device"),
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
