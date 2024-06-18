use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};
use std::time::SystemTime;

use bytes::Bytes;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tokio::sync::Semaphore;
use tracing::debug;

use crate::utils;

pub trait ArtifactGenerator {
	type Input;
	type ValidityKey: Eq + Serialize + DeserializeOwned;
	type Metadata: Serialize + DeserializeOwned;
	
	fn create_cache_key(&self, input: &Self::Input) -> String;
	
	async fn create_validity_key(&self, input: &Self::Input) -> anyhow::Result<Self::ValidityKey>;
	
	async fn generate_artifact(&self, input: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)>;
}

pub struct ArtifactCacheBuilder {
	cache_dir: Option<PathBuf>,
	task_limit: usize,
	file_size_limit: u64,
}

impl Default for ArtifactCacheBuilder {
	fn default() -> Self {
		Self {
			cache_dir: None,
			task_limit: 4,
			file_size_limit: u64::MAX,
		}
	}
}

impl ArtifactCacheBuilder {
	pub fn cache_dir(mut self, cache_dir: PathBuf) -> Self {
		self.cache_dir = Some(cache_dir);
		self
	}
	
	pub fn task_limit(mut self, limit: usize) -> Self {
		self.task_limit = limit;
		self
	}
	
	pub fn file_size_limit(mut self, limit: u64) -> Self {
		self.file_size_limit = limit;
		self
	}
	
	pub async fn build<G: ArtifactGenerator>(self, generator: G) -> Result<ArtifactCache<G>, io::Error> {
		ArtifactCache::init(generator, self.cache_dir.expect("No cache dir set"), self.task_limit, self.file_size_limit).await
	}
}

const ENTRY_METADATA_EXTENSION: &str = "meta.json";

pub struct ArtifactCache<G> {
	generator: G,
	cache_dir: PathBuf,
	locks: LockPool<CacheEntry>,
	lru_state: Mutex<LruState>,
	generation_limiter: Semaphore,
}

impl ArtifactCache<()> {
	pub fn builder() -> ArtifactCacheBuilder {
		ArtifactCacheBuilder::default()
	}
}

impl<G: ArtifactGenerator> ArtifactCache<G> {
	async fn init(generator: G, cache_dir: PathBuf, task_limit: usize, file_size_limit: u64) -> Result<Self, io::Error> {
		tokio::fs::create_dir_all(&cache_dir).await?;
		
		let mut read_dir = tokio::fs::read_dir(&cache_dir).await?;
		let mut entries = Vec::new();
		
		while let Some(dir_entry) = read_dir.next_entry().await? {
			let path = dir_entry.path();
			
			let Some(cache_key) = path.file_name()
				.and_then(OsStr::to_str)
				.and_then(|name| name.strip_suffix(".meta.json"))
			else { continue };
			
			let cache_entry_path = cache_dir.join(cache_key);
			
			let entry_metadata = tokio::fs::read(&path).await.ok()
				.and_then(|data| serde_json::from_slice::<CacheEntryMetadata<G::ValidityKey, G::Metadata>>(&data).ok());
			
			if let Some(entry_metadata) = entry_metadata {
				if tokio::fs::try_exists(&cache_entry_path).await.unwrap_or(false) {
					entries.push(LruEntry {
						key: cache_key.to_owned(),
						entry_size: entry_metadata.entry_size,
						last_accessed: entry_metadata.last_accessed,
					})
				}
			} else {
				// If the metadata is invalid then remove the cache entry
				// Otherwise it'll go untracked forever
				let _ = tokio::fs::remove_file(&path).await;
				let _ = tokio::fs::remove_file(&cache_entry_path).await;
			}
		}
		
		Ok(Self {
			generator,
			cache_dir,
			locks: LockPool::new(),
			lru_state: Mutex::new(LruState::new(entries, file_size_limit)),
			generation_limiter: Semaphore::new(task_limit),
		})
	}
	
	pub async fn get(&self, input: &G::Input) -> anyhow::Result<Option<CacheQuery<G::Metadata>>> {
		let validity_key = self.generator.create_validity_key(input).await?;
		let cache_key = self.generator.create_cache_key(input);
		
		let entry_lock = self.get_entry_lock(&cache_key);
		let entry = entry_lock.lock().await;
		
		match self.get_inner(&entry, &validity_key).await? {
			QueryResult::Valid(cache_query) => Ok(Some(cache_query)),
			QueryResult::Invalid => Ok(None),
		}
	}
	
	pub async fn get_or_generate(&self, input: G::Input) -> anyhow::Result<CacheQuery<G::Metadata>> {
		let validity_key = self.generator.create_validity_key(&input).await?;
		let cache_key = self.generator.create_cache_key(&input);
		
		let entry_lock = self.get_entry_lock(&cache_key);
		let entry = entry_lock.lock().await;
		
		match self.get_inner(&entry, &validity_key).await? {
			QueryResult::Valid(cache_query) => Ok(cache_query),
			QueryResult::Invalid => {
				let (artifact_data, metadata) = {
					let _permit = self.generation_limiter.acquire().await.unwrap();
					self.generator.generate_artifact(input).await?
				};
				
				let now = SystemTime::now();
				
				let entry_metadata = CacheEntryMetadata {
					cache_key: cache_key.clone(),
					creation_date: now,
					last_accessed: now,
					entry_size: artifact_data.len() as u64,
					validity_key,
					extra_metadata: metadata,
				};
				
				tokio::fs::write(&entry.cache_file_path, &artifact_data).await?;
				tokio::fs::write(&entry.cache_metadata_path, serde_json::to_vec_pretty(&entry_metadata).unwrap()).await?;
				
				let evicted = self.lru_state.lock().unwrap().insert(entry.cache_key.clone(), artifact_data.len() as u64);
				
				drop(entry);
				
				for evicted_cache_key in evicted {
					debug!("Evicting cache entry {} from {} cache", evicted_cache_key, std::any::type_name::<G>());
					
					let evicted_entry_lock = self.get_entry_lock(&evicted_cache_key);
					let evicted_entry = evicted_entry_lock.lock().await;
					
					let _ = tokio::fs::remove_file(&evicted_entry.cache_file_path).await;
					let _ = tokio::fs::remove_file(&evicted_entry.cache_metadata_path).await;
				}
				
				Ok(CacheQuery {
					entry_data: artifact_data,
					creation_date: now,
					metadata: entry_metadata.extra_metadata,
				})
			}
		}
	}
	
	async fn get_inner(&self, entry: &CacheEntry, validity_key: &G::ValidityKey) -> anyhow::Result<QueryResult<G::Metadata>> {
		if tokio::fs::metadata(&entry.cache_file_path).await.is_ok() {
			let entry_metadata = tokio::fs::read(&entry.cache_metadata_path).await.ok()
				.and_then(|data| serde_json::from_slice::<CacheEntryMetadata<G::ValidityKey, G::Metadata>>(&data).ok());
			
			if let Some(mut entry_metadata) = entry_metadata {
				if &entry_metadata.validity_key == validity_key {
					self.lru_state.lock().unwrap().promote(&entry.cache_key);
					
					entry_metadata.last_accessed = SystemTime::now();
					tokio::fs::write(&entry.cache_metadata_path, serde_json::to_vec_pretty(&entry_metadata).unwrap()).await?;
					
					let entry_data = tokio::fs::read(&entry.cache_file_path).await?;
					
					let cache_query = CacheQuery {
						entry_data: Bytes::from(entry_data),
						creation_date: entry_metadata.creation_date,
						metadata: entry_metadata.extra_metadata,
					};
					
					return Ok(QueryResult::Valid(cache_query));
				}
			}
		}
		
		return Ok(QueryResult::Invalid);
	}
	
	fn get_entry_lock(&self, cache_key: &str) -> Arc<tokio::sync::Mutex<CacheEntry>> {
		self.locks.get_lock(cache_key, || {
			let cache_file_path = self.cache_dir.join(cache_key);
			let cache_metadata_path = utils::add_extension(&cache_file_path, ENTRY_METADATA_EXTENSION);
			
			CacheEntry {
				cache_key: cache_key.to_owned(),
				cache_file_path,
				cache_metadata_path,
			}
		})
	}
}

#[derive(Debug, Eq, PartialEq)]
pub struct CacheQuery<M = ()> {
	pub entry_data: Bytes,
	pub creation_date: SystemTime,
	pub metadata: M,
}

#[derive(Debug, Eq, PartialEq)]
struct CacheEntry {
	cache_key: String,
	cache_file_path: PathBuf,
	cache_metadata_path: PathBuf,
}

#[derive(Debug, Eq, PartialEq)]
enum QueryResult<M> {
	Valid(CacheQuery<M>),
	Invalid,
}

#[derive(Eq, PartialEq, Serialize, Deserialize)]
struct CacheEntryMetadata<V, M> {
	cache_key: String,
	creation_date: SystemTime,
	last_accessed: SystemTime,
	entry_size: u64,
	validity_key: V,
	extra_metadata: M,
}

struct LockPool<T> {
	locks: Mutex<HashMap<String, Weak<tokio::sync::Mutex<T>>>>,
}

impl<T> LockPool<T> {
	pub fn new() -> Self {
		Self {
			locks: Mutex::new(HashMap::new()),
		}
	}
	
	fn get_lock(&self, key: &str, value_func: impl FnOnce() -> T) -> Arc<tokio::sync::Mutex<T>> {
		let mut locks = self.locks.lock().unwrap();
		
		if let Some(lock) = locks.get(key).and_then(Weak::upgrade) {
			return lock;
		}
		
		let new_lock = Arc::new(tokio::sync::Mutex::new(value_func()));
		locks.insert(key.to_owned(), Arc::downgrade(&new_lock));
		
		if locks.len() > 10 {
			locks.retain(|_, arc| arc.strong_count() > 0);
		}
		
		new_lock
	}
}

struct LruState {
	lru: LruCache<String, u64>,
	total_size: u64,
	size_limit: u64,
}

#[derive(Debug, Eq, PartialEq)]
struct LruEntry {
	key: String,
	entry_size: u64,
	last_accessed: SystemTime,
}

impl LruState {
	pub fn new(mut entries: Vec<LruEntry>, size_limit: u64) -> Self {
		let mut lru = LruCache::unbounded();
		let mut total_size = 0;
		
		// Sort from oldest to newest
		entries.sort_by_key(|entry| entry.last_accessed);
		
		for entry in entries {
			lru.put(entry.key, entry.entry_size);
			
			total_size += entry.entry_size;
		}
		
		Self {
			lru,
			total_size,
			size_limit,
		}
	}
	
	pub fn promote(&mut self, key: &str) {
		self.lru.promote(key);
	}
	
	pub fn insert(&mut self, key: String, entry_size: u64) -> Vec<String> {
		let old_entry_size = self.lru.put(key, entry_size).unwrap_or(0);
		
		self.total_size -= old_entry_size;
		self.total_size += entry_size;
		
		let mut removed = Vec::new();
		
		while self.total_size > self.size_limit {
			let (removed_key, removed_size) = self.lru.pop_lru().expect("Not entries left but total size is non zero");
			
			removed.push(removed_key);
			self.total_size -= removed_size;
		}
		
		removed
	}
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

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::task::Poll;
	use std::time::{Duration, SystemTime};
	
	use bytes::Bytes;
	use futures_util::poll;
	use tempfile::TempDir;
	
	use crate::web_server::services::artifact_cache::{ArtifactCache, ArtifactGenerator, CacheEntryMetadata, CacheQuery, ENTRY_METADATA_EXTENSION, LockPool, LruEntry, LruState};
	
	#[test]
	fn test_lru() {
		let lru_entries = vec![
			LruEntry {
				key: "key1".to_owned(),
				entry_size: 10,
				last_accessed: SystemTime::UNIX_EPOCH + Duration::from_secs(1),
			},
			LruEntry {
				key: "key3".to_owned(),
				entry_size: 10,
				last_accessed: SystemTime::UNIX_EPOCH + Duration::from_secs(3),
			},
			LruEntry {
				key: "key2".to_owned(),
				entry_size: 20,
				last_accessed: SystemTime::UNIX_EPOCH + Duration::from_secs(2),
			},
			LruEntry {
				key: "key4".to_owned(),
				entry_size: 10,
				last_accessed: SystemTime::UNIX_EPOCH + Duration::from_secs(4),
			},
		];
		
		let mut lru_state = LruState::new(lru_entries, u64::MAX);
		
		assert_eq!(lru_state.lru.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key4", "key3", "key2", "key1"]);
		assert_eq!(lru_state.total_size, 50);
		assert_eq!(lru_state.size_limit, u64::MAX);
		
		assert!(lru_state.insert("key5".to_owned(), 50).is_empty());
		
		assert_eq!(lru_state.lru.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key5", "key4", "key3", "key2", "key1"]);
		assert_eq!(lru_state.total_size, 100);
		assert_eq!(lru_state.size_limit, u64::MAX);
		
		assert!(lru_state.insert("key4".to_owned(), 30).is_empty());
		
		assert_eq!(lru_state.lru.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key4", "key5", "key3", "key2", "key1"]);
		assert_eq!(lru_state.total_size, 120);
		assert_eq!(lru_state.size_limit, u64::MAX);
		
		lru_state.promote("key3");
		
		assert_eq!(lru_state.lru.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key3", "key4", "key5", "key2", "key1"]);
		assert_eq!(lru_state.total_size, 120);
		assert_eq!(lru_state.size_limit, u64::MAX);
		
		lru_state.size_limit = 90;
		
		assert_eq!(lru_state.size_limit, 90);
		
		lru_state.promote("key5");
		
		assert_eq!(lru_state.lru.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key5", "key3", "key4", "key2", "key1"]);
		assert_eq!(lru_state.total_size, 120);
		assert_eq!(lru_state.size_limit, 90);
		
		assert_eq!(lru_state.insert("key6".to_owned(), 10), &["key1", "key2", "key4"]);
		
		assert_eq!(lru_state.lru.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key6", "key5", "key3"]);
		assert_eq!(lru_state.total_size, 70);
		assert_eq!(lru_state.size_limit, 90);
		
		assert_eq!(lru_state.lru.iter().map(|e| e.1).sum::<u64>(), lru_state.total_size);
	}
	
	#[tokio::test]
	async fn test_lock_pool() {
		let mut lock_pool: LockPool<u32> = LockPool::new();
		
		assert!(lock_pool.locks.get_mut().unwrap().is_empty());
		
		assert_eq!(*lock_pool.get_lock("key1", || 1).lock().await, 1);
		assert_eq!(lock_pool.locks.get_mut().unwrap().len(), 1);
		
		assert_eq!(*lock_pool.get_lock("key1", || 1).lock().await, 1);
		assert_eq!(lock_pool.locks.get_mut().unwrap().len(), 1);
		
		assert_eq!(*lock_pool.get_lock("key3", || 3).lock().await, 3);
		assert_eq!(lock_pool.locks.get_mut().unwrap().len(), 2);
		
		let lock = lock_pool.get_lock("key5", || 5);
		let lock_guard = lock.lock().await;
		
		assert_eq!(*lock_guard, 5);
		assert_eq!(lock_pool.locks.get_mut().unwrap().len(), 3);
		
		let lock_pool = Arc::new(lock_pool);
		let lock_pool2 = lock_pool.clone();
		
		let mut task = tokio::spawn(async move {
			let lock = lock_pool2.get_lock("key5", || 5);
			let lock_guard = lock.lock().await;
			
			*lock_guard
		});
		
		assert!(matches!(poll!(&mut task), Poll::Pending));
		tokio::task::yield_now().await;
		assert!(matches!(poll!(&mut task), Poll::Pending));
		
		drop(lock_guard);
		
		tokio::task::yield_now().await;
		assert!(matches!(poll!(&mut task), Poll::Ready(Ok(5))));
	}
	
	struct TestGenerator;
	
	impl ArtifactGenerator for TestGenerator {
		type Input = u32;
		type ValidityKey = ();
		type Metadata = String;
		
		fn create_cache_key(&self, input: &Self::Input) -> String {
			format!("key{}", *input)
		}
		
		async fn create_validity_key(&self, _input: &Self::Input) -> anyhow::Result<Self::ValidityKey> {
			Ok(())
		}
		
		async fn generate_artifact(&self, input: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)> {
			let data = format!("stuff{}", input);
			let metadata = format!("meta{}", input);
			
			Ok((data.into(), metadata))
		}
	}
	
	#[tokio::test]
	async fn test_artifact_cache() {
		let temp_dir = TempDir::new().unwrap();
		
		async fn write_entry(temp_dir: &TempDir, id: u32, time: u64) {
			let key = format!("key{}", id);
			let content = format!("stuff{}", id);
			
			tokio::fs::write(temp_dir.path().join(&key), content.clone()).await.unwrap();
			
			tokio::fs::write(temp_dir.path().join(&key).with_extension(ENTRY_METADATA_EXTENSION), serde_json::to_vec_pretty(&CacheEntryMetadata {
				cache_key: key.clone(),
				creation_date: SystemTime::UNIX_EPOCH + Duration::from_secs(time / 10),
				last_accessed: SystemTime::UNIX_EPOCH + Duration::from_secs(time),
				entry_size: content.len() as u64,
				validity_key: (),
				extra_metadata: format!("meta{}", id),
			}).unwrap()).await.unwrap();
		}
		
		write_entry(&temp_dir, 1, 10).await;
		write_entry(&temp_dir, 3, 30).await;
		write_entry(&temp_dir, 2, 20).await;
		
		let mut artifact_cache = ArtifactCache::builder()
			.cache_dir(temp_dir.path().to_owned())
			.build(TestGenerator)
			.await.unwrap();
		
		{
			let lru_state = artifact_cache.lru_state.get_mut().unwrap();
			assert_eq!(lru_state.lru.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key3", "key2", "key1"]);
			assert_eq!(lru_state.total_size, 18);
			assert_eq!(lru_state.size_limit, u64::MAX);
		}
		
		assert_eq!(artifact_cache.get(&99).await.unwrap(), None);
		
		assert_eq!(artifact_cache.get(&1).await.unwrap(), Some(CacheQuery {
			entry_data: "stuff1".into(),
			creation_date: SystemTime::UNIX_EPOCH + Duration::from_secs(1),
			metadata: "meta1".into(),
		}));
		
		{
			let lru_state = artifact_cache.lru_state.get_mut().unwrap();
			assert_eq!(lru_state.lru.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key1", "key3", "key2"]);
			assert_eq!(lru_state.total_size, 18);
			assert_eq!(lru_state.size_limit, u64::MAX);
		}
		
		let now = SystemTime::now();
		let query = artifact_cache.get_or_generate(44).await.unwrap();
		
		assert_eq!(query.entry_data, "stuff44");
		assert_eq!(query.metadata, "meta44");
		assert!(query.creation_date > now);
		
		{
			let lru_state = artifact_cache.lru_state.get_mut().unwrap();
			assert_eq!(lru_state.lru.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key44", "key1", "key3", "key2"]);
			assert_eq!(lru_state.total_size, 25);
			assert_eq!(lru_state.size_limit, u64::MAX);
		}
		
		assert_eq!(tokio::fs::read(temp_dir.path().join("key44")).await.unwrap(), "stuff44".as_bytes());
		
		let mut artifact_cache = ArtifactCache::builder()
			.cache_dir(temp_dir.path().to_owned())
			.build(TestGenerator)
			.await.unwrap();
		
		let query = artifact_cache.get(&44).await.unwrap().unwrap();
		
		assert_eq!(query.entry_data, "stuff44");
		assert_eq!(query.metadata, "meta44");
		assert!(query.creation_date > now);
		
		{
			let lru_state = artifact_cache.lru_state.get_mut().unwrap();
			assert_eq!(lru_state.lru.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key44", "key1", "key3", "key2"]);
			assert_eq!(lru_state.total_size, 25);
			assert_eq!(lru_state.size_limit, u64::MAX);
		}
		
		artifact_cache.lru_state.get_mut().unwrap().size_limit = 20;
		
		assert!(tokio::fs::try_exists(temp_dir.path().join("key2")).await.unwrap());
		assert!(tokio::fs::try_exists(temp_dir.path().join("key2").with_extension(ENTRY_METADATA_EXTENSION)).await.unwrap());
		assert!(tokio::fs::try_exists(temp_dir.path().join("key3")).await.unwrap());
		assert!(tokio::fs::try_exists(temp_dir.path().join("key3").with_extension(ENTRY_METADATA_EXTENSION)).await.unwrap());
		
		let now = SystemTime::now();
		let query = artifact_cache.get_or_generate(5).await.unwrap();
		
		assert_eq!(query.entry_data, "stuff5");
		assert_eq!(query.metadata, "meta5");
		assert!(query.creation_date > now);
		
		{
			let lru_state = artifact_cache.lru_state.get_mut().unwrap();
			assert_eq!(lru_state.lru.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key5", "key44", "key1"]);
			assert_eq!(lru_state.total_size, 19);
			assert_eq!(lru_state.size_limit, 20);
		}
		
		assert!(!tokio::fs::try_exists(temp_dir.path().join("key2")).await.unwrap());
		assert!(!tokio::fs::try_exists(temp_dir.path().join("key2").with_extension(ENTRY_METADATA_EXTENSION)).await.unwrap());
		assert!(!tokio::fs::try_exists(temp_dir.path().join("key3")).await.unwrap());
		assert!(!tokio::fs::try_exists(temp_dir.path().join("key3").with_extension(ENTRY_METADATA_EXTENSION)).await.unwrap());
	}
}
