use std::ops::{Deref, DerefMut};
use ffmpeg_sys_next::{av_buffer_ref, av_buffer_unref, av_hwdevice_ctx_create, AVBufferRef, AVHWDeviceType};
use std::ptr::{null, null_mut};
use std::sync::{Arc, Mutex};

use crate::media_manipulation::media_utils::{av_error, check_alloc};

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

pub struct DevicePool {
	inner: Mutex<DevicePoolInner>,
}

struct DevicePoolInner {
	devices: Vec<HardwareDeviceContext>,
	factory: Box<dyn Fn() -> anyhow::Result<HardwareDeviceContext> + Send>,
}

impl DevicePool {
	pub fn new(factory: impl Fn() -> anyhow::Result<HardwareDeviceContext> + Send + 'static) -> Arc<Self> {
		Arc::new(Self {
			inner: Mutex::new(DevicePoolInner {
				devices: Vec::new(),
				factory: Box::new(factory),
			}),
		})
	}
	
	pub fn take_device(self: &Arc<Self>) -> anyhow::Result<BorrowedDevice> {
		let mut inner = self.inner.lock().expect("Lock poisoned");
		
		let device = match inner.devices.pop() {
			Some(device) => device,
			None => (inner.factory)()?,
		};
		
		Ok(BorrowedDevice {
			device: Some(device),
			pool: self.clone(),
		})
	}
}

pub struct BorrowedDevice {
	device: Option<HardwareDeviceContext>,
	pool: Arc<DevicePool>,
}

impl Deref for BorrowedDevice {
	type Target = HardwareDeviceContext;
	
	fn deref(&self) -> &Self::Target {
		self.device.as_ref().unwrap()
	}
}

impl DerefMut for BorrowedDevice {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.device.as_mut().unwrap()
	}
}

impl Drop for BorrowedDevice {
	fn drop(&mut self) {
		let mut inner = self.pool.inner.lock().unwrap();
		inner.devices.push(self.device.take().unwrap());
	}
}