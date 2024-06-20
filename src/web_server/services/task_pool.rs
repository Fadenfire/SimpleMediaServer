use std::future::Future;

use tokio::sync::Semaphore;

pub struct TaskPool {
	limiter: Semaphore,
}

impl TaskPool {
	pub fn new(task_limit: usize) -> Self {
		Self {
			limiter: Semaphore::new(task_limit),
		}
	}
	
	pub async fn execute_task<T>(&self, task: impl Future<Output = T>) -> T {
		let _permit = self.limiter.acquire().await.unwrap();
		task.await
	}
}
