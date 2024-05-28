use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};
use std::time::SystemTime;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tokio::sync::Semaphore;

pub trait ArtifactGenerator {
	type Input;
	type ValidityKey: Eq + Serialize + DeserializeOwned;
	type Metadata: Serialize + DeserializeOwned;
	
	fn create_cache_key(&self, input: &Self::Input) -> String;
	
	async fn create_validity_key(&self, input: &Self::Input) -> anyhow::Result<Self::ValidityKey>;
	
	async fn generate_artifact(&self, input: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)>;
}

pub struct ArtifactCache<G: ArtifactGenerator> {
	generator: G,
	cache_dir: PathBuf,
	locks: Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>,
	generation_limiter: Semaphore,
}

impl<G: ArtifactGenerator> ArtifactCache<G> {
	pub async fn new(generator: G, cache_dir: PathBuf) -> Result<Self, io::Error> {
		tokio::fs::create_dir_all(&cache_dir).await?;
		
		Ok(Self {
			generator,
			cache_dir,
			locks: Mutex::new(HashMap::new()),
			generation_limiter: Semaphore::new(64),
		})
	}
	
	pub fn with_task_limit(mut self, limit: usize) -> Self {
		self.generation_limiter = Semaphore::new(limit);
		self
	}
	
	pub async fn get(&self, input: &G::Input) -> anyhow::Result<Option<CacheEntry<G::Metadata>>> {
		let validity_key = self.generator.create_validity_key(input).await?;
		
		let cache_key = self.generator.create_cache_key(input);
		let lock = self.get_lock(&cache_key);
		let _lock_guard = lock.lock().await;
		
		match self.get_inner(&cache_key, &validity_key).await? {
			QueryResult::Present { cache_entry } => Ok(Some(cache_entry)),
			QueryResult::Absent { .. } => Ok(None),
		}
	}
	
	pub async fn get_or_generate(&self, input: G::Input) -> anyhow::Result<CacheEntry<G::Metadata>> {
		let validity_key = self.generator.create_validity_key(&input).await?;
		
		let cache_key = self.generator.create_cache_key(&input);
		let lock = self.get_lock(&cache_key);
		let _lock_guard = lock.lock().await;
		
		match self.get_inner(&cache_key, &validity_key).await? {
			QueryResult::Present { cache_entry } => Ok(cache_entry),
			QueryResult::Absent { cache_file_path, cache_metadata_path } => {
				let (artifact_data, metadata) = {
					let _permit = self.generation_limiter.acquire().await.unwrap();
					self.generator.generate_artifact(input).await?
				};
				
				let entry_metadata = CacheEntryMetadata {
					validity_key,
					metadata,
				};
				
				tokio::fs::write(&cache_file_path, &artifact_data).await?;
				tokio::fs::write(&cache_metadata_path, serde_json::to_vec_pretty(&entry_metadata).unwrap()).await?;
				
				Ok(CacheEntry {
					cache_file: cache_file_path,
					mod_time: SystemTime::now(),
					metadata: entry_metadata.metadata
				})
			}
		}
	}
	
	async fn get_inner(&self, cache_key: &str, validity_key: &G::ValidityKey) -> anyhow::Result<QueryResult<G::Metadata>> {
		let cache_file_path = self.cache_dir.join(cache_key);
		let cache_metadata_path = cache_file_path.with_extension("meta.json");
		
		if let Some(file_metadata) = tokio::fs::metadata(&cache_file_path).await.ok() {
			let entry_metadata = tokio::fs::read(&cache_metadata_path).await.ok()
				.and_then(|data| serde_json::from_slice::<CacheEntryMetadata<G::ValidityKey, G::Metadata>>(&data).ok());
			
			if let Some(entry_metadata) = entry_metadata {
				if &entry_metadata.validity_key == validity_key {
					let cache_entry = CacheEntry {
						cache_file: cache_file_path,
						mod_time: file_metadata.modified().expect("System/FS does not support mod time"),
						metadata: entry_metadata.metadata
					};
					
					return Ok(QueryResult::Present { cache_entry });
				}
			}
		}
		
		return Ok(QueryResult::Absent {
			cache_file_path,
			cache_metadata_path,
		})
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

pub struct CacheEntry<M = ()> {
	pub cache_file: PathBuf,
	pub mod_time: SystemTime,
	pub metadata: M,
}

enum QueryResult<M> {
	Present {
		cache_entry: CacheEntry<M>,
	},
	Absent {
		cache_file_path: PathBuf,
		cache_metadata_path: PathBuf,
	}
}

#[derive(Eq, PartialEq, Serialize, Deserialize)]
struct CacheEntryMetadata<V, M> {
	validity_key: V,
	metadata: M,
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