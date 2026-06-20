use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

pub struct ResourcePool<T> {
	inner: Mutex<ResourcePoolInner<T>>,
}

struct ResourcePoolInner<T> {
	devices: Vec<T>,
	factory: Box<dyn Fn() -> anyhow::Result<T> + Send>,
}

impl<T> ResourcePool<T> {
	pub fn new(factory: impl Fn() -> anyhow::Result<T> + Send + 'static) -> Arc<Self> {
		Arc::new(Self {
			inner: Mutex::new(ResourcePoolInner {
				devices: Vec::new(),
				factory: Box::new(factory),
			}),
		})
	}
	
	pub fn take(self: &Arc<Self>) -> anyhow::Result<BorrowedResource<T>> {
		let mut inner = self.inner.lock().expect("Lock poisoned");
		
		let device = match inner.devices.pop() {
			Some(device) => device,
			None => (inner.factory)()?,
		};
		
		Ok(BorrowedResource {
			resource: Some(device),
			pool: self.clone(),
		})
	}
}

pub struct BorrowedResource<T> {
	resource: Option<T>,
	pool: Arc<ResourcePool<T>>,
}

impl<T> Deref for BorrowedResource<T> {
	type Target = T;
	
	fn deref(&self) -> &Self::Target {
		self.resource.as_ref().unwrap()
	}
}

impl<T> DerefMut for BorrowedResource<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.resource.as_mut().unwrap()
	}
}

impl<T> Drop for BorrowedResource<T> {
	fn drop(&mut self) {
		let mut inner = self.pool.inner.lock().unwrap();
		inner.devices.push(self.resource.take().unwrap());
	}
}