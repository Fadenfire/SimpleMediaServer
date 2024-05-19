use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};
use std::time::SystemTime;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

pub struct ArtifactCache<V> {
	cache_dir: PathBuf,
	locks: Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>,
	phantom_data: PhantomData<V>,
}

impl<V: Eq + Serialize + DeserializeOwned> ArtifactCache<V> {
	pub async fn new(cache_dir: PathBuf) -> Result<Self, io::Error> {
		tokio::fs::create_dir_all(&cache_dir).await?;
		
		Ok(Self {
			cache_dir,
			locks: Mutex::new(HashMap::new()),
			phantom_data: PhantomData,
		})
	}
	
	pub async fn get_or_insert<F, Fut>(&self, cache_key: &str, validity_key: V, func: F) -> anyhow::Result<CacheEntry>
	where
		F: FnOnce() -> Fut,
		Fut: Future<Output = anyhow::Result<Bytes>>
	{
		let lock = self.get_lock(cache_key);
		let _lock_guard = lock.lock().await;
		
		let cache_file_path = self.cache_dir.join(cache_key);
		let cache_metadata_path = cache_file_path.with_extension("meta.json");
		
		if let Some(file_metadata) = tokio::fs::metadata(&cache_file_path).await.ok() {
			let entry_metadata = tokio::fs::read(&cache_metadata_path).await.ok()
				.and_then(|data| serde_json::from_slice::<CacheEntryMetadata<V>>(&data).ok());
			
			if let Some(entry_metadata) = entry_metadata {
				if entry_metadata.validity_key == validity_key {
					return Ok(CacheEntry {
						cache_file: cache_file_path,
						mod_time: file_metadata.modified().expect("System/FS does not support mod time"),
					});
				}
			}
		}
		
		let artifact_data = func().await?;
		
		let entry_metadata = CacheEntryMetadata {
			validity_key,
		};
		
		tokio::fs::write(&cache_file_path, &artifact_data).await?;
		tokio::fs::write(&cache_metadata_path, serde_json::to_vec_pretty(&entry_metadata).unwrap()).await?;
		
		return Ok(CacheEntry {
			cache_file: cache_file_path,
			mod_time: SystemTime::now(),
		});
	}
	
	fn get_lock(&self, key: &str) -> Arc<tokio::sync::Mutex<()>> {
		let mut locks = self.locks.lock().unwrap();
		
		if let Some(lock) = locks.get(key).and_then(Weak::upgrade) {
			return lock;
		}
		
		let new_lock = Arc::new(tokio::sync::Mutex::new(()));
		locks.insert(key.to_owned(), Arc::downgrade(&new_lock));
		
		if locks.len() > 10 {
			locks.retain(|_, arc| arc.strong_count() > 0);
		}
		
		new_lock
	}
}

pub struct CacheEntry {
	pub cache_file: PathBuf,
	pub mod_time: SystemTime,
}

#[derive(Eq, PartialEq, Serialize, Deserialize)]
struct CacheEntryMetadata<V> {
	validity_key: V,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileValidityKey {
	source_path: PathBuf,
	file_size: u64,
	mod_time: Option<SystemTime>,
}

impl FileValidityKey {
	pub async fn from_file(source_path: impl AsRef<Path>) -> anyhow::Result<Self> {
		let metadata = tokio::fs::metadata(&source_path).await?;
		
		Ok(Self {
			source_path: source_path.as_ref().to_owned(),
			file_size: metadata.len(),
			mod_time: metadata.modified().ok(),
		})
	}
}