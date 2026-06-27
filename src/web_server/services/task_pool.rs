use std::future::Future;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub struct TaskPool {
	limiter: Arc<Semaphore>,
}

impl TaskPool {
	pub fn new(task_limit: usize) -> Self {
		Self {
			limiter: Arc::new(Semaphore::new(task_limit)),
		}
	}
	
	pub async fn reserve(&self) -> ReservedTask {
		let permit = self.limiter.clone()
			.acquire_owned()
			.await
			.unwrap();
		
		ReservedTask { permit }
	}
}

pub struct ReservedTask {
	permit: OwnedSemaphorePermit,
}

impl ReservedTask {
	pub async fn execute_task<T>(self, task: impl Future<Output = T>) -> T {
		task.await
	}
}
