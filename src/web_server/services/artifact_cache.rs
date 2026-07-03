use bytes::Bytes;
use hashlink::LinkedHashMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};
use time::OffsetDateTime;
use tracing::debug;

use crate::utils;
use crate::web_server::services::task_pool::{ReservedTask, TaskPool};

pub trait ArtifactGenerator {
	type Input;
	type Metadata: Clone + Serialize + DeserializeOwned;
	
	async fn create_cache_key(&self, input: &Self::Input) -> anyhow::Result<String>;
	
	async fn generate_artifact(&self, input: Self::Input) -> anyhow::Result<(Bytes, Self::Metadata)>;
}

pub fn builder() -> ArtifactCacheBuilder {
	ArtifactCacheBuilder::default()
}

pub struct ArtifactCacheBuilder {
	cache_dir: Option<PathBuf>,
	task_pool: Option<Arc<TaskPool>>,
	file_size_limit: u64,
}

impl Default for ArtifactCacheBuilder {
	fn default() -> Self {
		Self {
			cache_dir: None,
			task_pool: None,
			file_size_limit: u64::MAX,
		}
	}
}

impl ArtifactCacheBuilder {
	pub fn cache_dir(mut self, cache_dir: PathBuf) -> Self {
		self.cache_dir = Some(cache_dir);
		self
	}
	
	pub fn task_pool(mut self, task_pool: Arc<TaskPool>) -> Self {
		self.task_pool = Some(task_pool);
		self
	}
	
	pub fn file_size_limit(mut self, limit: u64) -> Self {
		self.file_size_limit = limit;
		self
	}
	
	pub async fn build<G: ArtifactGenerator>(self, generator: G) -> Result<ArtifactCache<G>, io::Error> {
		ArtifactCache::init(
			generator,
			self.cache_dir.expect("No cache dir set"),
			self.task_pool.expect("No task pool set"),
			self.file_size_limit
		).await
	}
}

const ENTRY_METADATA_EXTENSION: &str = "meta.json";

pub struct ArtifactCache<G: ArtifactGenerator> {
	generator: G,
	cache_dir: PathBuf,
	locks: LockPool<HeldCacheEntryInner>,
	entry_tracker: Mutex<EntryTracker<G::Metadata>>,
	task_pool: Arc<TaskPool>,
}

impl<G: ArtifactGenerator> ArtifactCache<G> {
	async fn init(generator: G, cache_dir: PathBuf, task_pool: Arc<TaskPool>, file_size_limit: u64) -> Result<Self, io::Error> {
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
				.and_then(|data| serde_json::from_slice::<CacheEntryMetadata<G::Metadata>>(&data).ok());
			
			let mut valid = false;
			
			if let Some(entry_metadata) = entry_metadata {
				if tokio::fs::try_exists(&cache_entry_path).await.unwrap_or(false) {
					entries.push(entry_metadata);
					
					valid = true;
				}
			}
			
			if !valid {
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
			entry_tracker: Mutex::new(EntryTracker::new(entries, file_size_limit)),
			task_pool,
		})
	}
	
	pub async fn get(&self, input: &G::Input) -> anyhow::Result<Option<CacheQuery<G::Metadata>>> {
		let held_entry = self.lock_entry(&input).await?;
		
		self.get_inner(&held_entry).await
	}
	
	pub async fn get_or_reserve(&self, input: G::Input) -> anyhow::Result<QueryResult<'_, G>> {
		let held_entry = self.lock_entry(&input).await?;
		
		match self.get_inner(&held_entry).await? {
			Some(cache_query) => Ok(QueryResult::Valid(cache_query)),
			None => {
				let task_reservation = self.task_pool.reserve().await;
				
				Ok(QueryResult::Invalid(PendingGeneration {
					cache: self,
					input,
					held_entry,
					task_reservation,
				}))
			}
		}
	}
	
	pub async fn get_or_generate(&self, input: G::Input) -> anyhow::Result<CacheQuery<G::Metadata>> {
		self.get_or_reserve(input)
			.await?
			.unwrap_or_generate()
			.await
	}
	
	async fn get_inner(&self, held_entry: &HeldCacheEntry) -> anyhow::Result<Option<CacheQuery<G::Metadata>>> {
		if tokio::fs::metadata(&held_entry.cache_file_path).await.is_ok() {
			let entry_metadata = self.entry_tracker.lock().unwrap()
				.get_entry(&held_entry.cache_key);
			
			if let Some(entry_metadata) = entry_metadata {
				tokio::fs::write(&held_entry.cache_metadata_path, serde_json::to_vec_pretty(&entry_metadata).unwrap()).await?;
				
				let entry_data = tokio::fs::read(&held_entry.cache_file_path).await?;
				
				let cache_query = CacheQuery {
					entry_data: Bytes::from(entry_data),
					creation_date: entry_metadata.creation_date,
					metadata: entry_metadata.extra_metadata,
				};
				
				return Ok(Some(cache_query));
			}
		}
		
		Ok(None)
	}
	
	async fn lock_entry(&self, input: &G::Input) -> anyhow::Result<HeldCacheEntry> {
		let cache_key = self.generator.create_cache_key(&input).await?;
		
		Ok(self.get_entry_lock(&cache_key).lock_owned().await)
	}
	
	fn get_entry_lock(&self, cache_key: &str) -> Arc<tokio::sync::Mutex<HeldCacheEntryInner>> {
		self.locks.get_lock(&cache_key, || {
			let cache_file_path = self.cache_dir.join(&cache_key);
			let cache_metadata_path = utils::add_extension(&cache_file_path, ENTRY_METADATA_EXTENSION);
			
			HeldCacheEntryInner {
				cache_key: cache_key.to_owned(),
				cache_file_path,
				cache_metadata_path,
			}
		})
	}
	
	pub fn cache_size(&self) -> u64 {
		self.entry_tracker.lock().unwrap().total_size
	}
}

pub struct PendingGeneration<'a, G: ArtifactGenerator> {
	cache: &'a ArtifactCache<G>,
	input: G::Input,
	held_entry: HeldCacheEntry,
	task_reservation: ReservedTask,
}

impl<'a, G: ArtifactGenerator> PendingGeneration<'a, G> {
	pub async fn generate(self) -> anyhow::Result<CacheQuery<G::Metadata>> {
		let (artifact_data, metadata) =
			self.task_reservation.execute_task(self.cache.generator.generate_artifact(self.input)).await?;
		
		let now = OffsetDateTime::now_utc();
		
		let entry_metadata = CacheEntryMetadata {
			cache_key: self.held_entry.cache_key.clone(),
			creation_date: now,
			last_accessed: now,
			entry_size: artifact_data.len() as u64,
			extra_metadata: metadata,
		};
		
		tokio::fs::write(&self.held_entry.cache_file_path, &artifact_data).await?;
		tokio::fs::write(&self.held_entry.cache_metadata_path, serde_json::to_vec_pretty(&entry_metadata).unwrap()).await?;
		
		let to_evict = self.cache.entry_tracker.lock().unwrap()
			.insert(entry_metadata.clone());
		
		drop(self.held_entry);
		
		for evicted_cache_key in to_evict {
			let evicted_entry_lock = self.cache.get_entry_lock(&evicted_cache_key);
			
			// Try to evict the entry if it's not being used
			if let Ok(held_entry) = evicted_entry_lock.try_lock() {
				debug!("Evicting cache entry {} from {} cache", evicted_cache_key, std::any::type_name::<G>());
				
				self.cache.entry_tracker.lock().unwrap()
					.remove_entry(&evicted_cache_key);
				
				let _ = tokio::fs::remove_file(&held_entry.cache_file_path).await;
				let _ = tokio::fs::remove_file(&held_entry.cache_metadata_path).await;
			}
		}
		
		Ok(CacheQuery {
			entry_data: artifact_data,
			creation_date: now,
			metadata: entry_metadata.extra_metadata,
		})
	}
}

#[derive(Debug, Eq, PartialEq)]
pub struct CacheQuery<M = ()> {
	pub entry_data: Bytes,
	pub creation_date: OffsetDateTime,
	pub metadata: M,
}

pub enum QueryResult<'a, G: ArtifactGenerator> {
	Valid(CacheQuery<G::Metadata>),
	Invalid(PendingGeneration<'a, G>),
}

impl<'a, G: ArtifactGenerator> QueryResult<'a, G> {
	pub async fn unwrap_or_generate(self) -> anyhow::Result<CacheQuery<G::Metadata>> {
		match self {
			QueryResult::Valid(query) => Ok(query),
			QueryResult::Invalid(pending) => pending.generate().await,
		}
	}
}

type HeldCacheEntry = tokio::sync::OwnedMutexGuard<HeldCacheEntryInner>;

#[derive(Debug, Eq, PartialEq)]
struct HeldCacheEntryInner {
	cache_key: String,
	cache_file_path: PathBuf,
	cache_metadata_path: PathBuf,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
struct CacheEntryMetadata<M> {
	cache_key: String,
	#[serde(with = "time::serde::iso8601")]
	creation_date: OffsetDateTime,
	#[serde(with = "time::serde::iso8601")]
	last_accessed: OffsetDateTime,
	entry_size: u64,
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

struct EntryTracker<M> {
	entries: LinkedHashMap<String, CacheEntryMetadata<M>>,
	total_size: u64,
	size_limit: u64,
}

impl<M: Clone> EntryTracker<M> {
	pub fn new(mut entries_vec: Vec<CacheEntryMetadata<M>>, size_limit: u64) -> Self {
		let mut entries = LinkedHashMap::new();
		let mut total_size = 0;
		
		// Sort from oldest to newest
		entries_vec.sort_by_key(|entry| entry.last_accessed);
		
		for entry in entries_vec {
			total_size += entry.entry_size;
			
			entries.insert(entry.cache_key.clone(), entry);
		}
		
		Self {
			entries,
			total_size,
			size_limit,
		}
	}
	
	pub fn get_entry(&mut self, key: &str) -> Option<CacheEntryMetadata<M>> {
		self.entries.to_back(key)
			.map(|entry| {
				entry.last_accessed = OffsetDateTime::now_utc();
				
				entry.clone()
			})
	}
	
	pub fn remove_entry(&mut self, key: &str) -> Option<CacheEntryMetadata<M>> {
		let removed = self.entries.remove(key)?;
		self.total_size -= removed.entry_size;
		
		Some(removed)
	}
	
	pub fn insert(&mut self, new_entry: CacheEntryMetadata<M>) -> Vec<String> {
		self.total_size += new_entry.entry_size;
		
		if let Some(old_entry) = self.entries.insert(new_entry.cache_key.clone(), new_entry) {
			self.total_size -= old_entry.entry_size;
		}
		
		let mut to_remove = Vec::new();
		let mut future_size = self.total_size;
		
		for entry in self.entries.values() {
			if future_size <= self.size_limit {
				break;
			}
			
			to_remove.push(entry.cache_key.clone());
			future_size -= entry.entry_size;
		}
		
		to_remove
	}
}

pub async fn create_file_metadata_hash(file_path: &Path) -> anyhow::Result<String> {
	let full_path = tokio::fs::canonicalize(file_path).await?;
	let metadata = tokio::fs::metadata(&full_path).await?;
	
	let mod_time = metadata.modified()?
		.duration_since(std::time::UNIX_EPOCH)?
		.as_nanos();
	
	let string = format!(
		"{}\0{}\0{}",
		full_path.as_os_str().to_string_lossy(),
		metadata.len(),
		mod_time,
	);
	
	Ok(blake3::hash(string.as_bytes()).to_hex().to_string())
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::task::Poll;
	use std::time::Duration;
	
	use bytes::Bytes;
	use futures_util::poll;
	use tempfile::TempDir;
	use time::macros::datetime;
	use time::OffsetDateTime;
	use crate::web_server::services::artifact_cache::{ArtifactGenerator, CacheEntryMetadata, CacheQuery, EntryTracker, LockPool, ENTRY_METADATA_EXTENSION};
	use crate::web_server::services::task_pool::TaskPool;
	
	#[test]
	fn test_lru() {
		fn make_entry(key: &str, entry_size: u64, last_accessed: OffsetDateTime) -> CacheEntryMetadata<()> {
			CacheEntryMetadata {
				cache_key: key.to_owned(),
				entry_size,
				creation_date: last_accessed.clone(),
				last_accessed,
				extra_metadata: ()
			}
		}
		
		let lru_entries = vec![
			make_entry("key1", 10, datetime!(2020-01-01 00:00:01 UTC)),
			make_entry("key3", 10, datetime!(2020-01-01 00:00:03 UTC)),
			make_entry("key2", 20, datetime!(2020-01-01 00:00:02 UTC)),
			make_entry("key4", 10, datetime!(2020-01-01 00:00:04 UTC)),
		];
		
		let mut lru_state = EntryTracker::new(lru_entries, u64::MAX);
		
		assert_eq!(lru_state.entries.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key1", "key2", "key3", "key4"]);
		assert_eq!(lru_state.total_size, 50);
		assert_eq!(lru_state.size_limit, u64::MAX);
		
		assert!(lru_state.insert(make_entry("key5", 50, datetime!(2020-01-01 00:00:05 UTC))).is_empty());
		
		assert_eq!(lru_state.entries.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key1", "key2", "key3", "key4", "key5"]);
		assert_eq!(lru_state.total_size, 100);
		assert_eq!(lru_state.size_limit, u64::MAX);
		
		assert!(lru_state.insert(make_entry("key4", 30, datetime!(2020-01-01 00:00:09 UTC))).is_empty());
		
		assert_eq!(lru_state.entries.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key1", "key2", "key3", "key5", "key4"]);
		assert_eq!(lru_state.total_size, 120);
		assert_eq!(lru_state.size_limit, u64::MAX);
		
		lru_state.get_entry("key3");
		
		assert_eq!(lru_state.entries.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key1", "key2", "key5", "key4", "key3"]);
		assert_eq!(lru_state.total_size, 120);
		assert_eq!(lru_state.size_limit, u64::MAX);
		
		lru_state.size_limit = 90;
		
		assert_eq!(lru_state.size_limit, 90);
		
		lru_state.get_entry("key5");
		
		assert_eq!(lru_state.entries.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key1", "key2", "key4", "key3", "key5"]);
		assert_eq!(lru_state.total_size, 120);
		assert_eq!(lru_state.size_limit, 90);
		
		let to_evict = lru_state.insert(make_entry("key6", 10, datetime!(2020-01-01 00:00:10 UTC)));
		assert_eq!(to_evict, &["key1", "key2", "key4"]);
		
		for key in &to_evict {
			lru_state.remove_entry(key);
		}

		assert_eq!(lru_state.entries.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key3", "key5", "key6"]);
		assert_eq!(lru_state.total_size, 70);
		assert_eq!(lru_state.size_limit, 90);

		assert_eq!(lru_state.entries.iter().map(|e| e.1.entry_size).sum::<u64>(), lru_state.total_size);
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
		type Metadata = String;

		async fn create_cache_key(&self, input: &Self::Input) -> anyhow::Result<String> {
			Ok(format!("key{}", *input))
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
				creation_date: datetime!(2020-01-01 00:00:00 UTC) + Duration::from_secs(time / 10),
				last_accessed: datetime!(2020-01-01 00:00:00 UTC) + Duration::from_secs(time),
				entry_size: content.len() as u64,
				extra_metadata: format!("meta{}", id),
			}).unwrap()).await.unwrap();
		}
		
		write_entry(&temp_dir, 1, 10).await;
		write_entry(&temp_dir, 3, 30).await;
		write_entry(&temp_dir, 2, 20).await;
		
		let task_pool = Arc::new(TaskPool::new(4));
		
		let mut artifact_cache = super::builder()
			.cache_dir(temp_dir.path().to_owned())
			.task_pool(task_pool.clone())
			.build(TestGenerator)
			.await.unwrap();
		
		{
			let lru_state = artifact_cache.entry_tracker.get_mut().unwrap();
			assert_eq!(lru_state.entries.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key1", "key2", "key3"]);
			assert_eq!(lru_state.total_size, 18);
			assert_eq!(lru_state.size_limit, u64::MAX);
		}
		
		assert_eq!(artifact_cache.get(&99).await.unwrap(), None);
		
		assert_eq!(artifact_cache.get(&1).await.unwrap(), Some(CacheQuery {
			entry_data: "stuff1".into(),
			creation_date: datetime!(2020-01-01 00:00:00 UTC) + Duration::from_secs(1),
			metadata: "meta1".into(),
		}));
		
		{
			let lru_state = artifact_cache.entry_tracker.get_mut().unwrap();
			assert_eq!(lru_state.entries.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key2", "key3", "key1"]);
			assert_eq!(lru_state.total_size, 18);
			assert_eq!(lru_state.size_limit, u64::MAX);
		}
		
		let now = OffsetDateTime::now_utc();
		let query = artifact_cache.get_or_generate(44).await.unwrap();
		
		assert_eq!(query.entry_data, "stuff44");
		assert_eq!(query.metadata, "meta44");
		assert!(query.creation_date > now);
		
		{
			let lru_state = artifact_cache.entry_tracker.get_mut().unwrap();
			assert_eq!(lru_state.entries.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key2", "key3", "key1", "key44"]);
			assert_eq!(lru_state.total_size, 25);
			assert_eq!(lru_state.size_limit, u64::MAX);
		}
		
		assert_eq!(tokio::fs::read(temp_dir.path().join("key44")).await.unwrap(), "stuff44".as_bytes());
		
		let mut artifact_cache = super::builder()
			.cache_dir(temp_dir.path().to_owned())
			.task_pool(task_pool.clone())
			.build(TestGenerator)
			.await.unwrap();
		
		let query = artifact_cache.get(&44).await.unwrap().unwrap();
		
		assert_eq!(query.entry_data, "stuff44");
		assert_eq!(query.metadata, "meta44");
		assert!(query.creation_date > now);
		
		{
			let lru_state = artifact_cache.entry_tracker.get_mut().unwrap();
			assert_eq!(lru_state.entries.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key2", "key3", "key1", "key44"]);
			assert_eq!(lru_state.total_size, 25);
			assert_eq!(lru_state.size_limit, u64::MAX);
		}
		
		artifact_cache.entry_tracker.get_mut().unwrap().size_limit = 20;
		
		assert!(tokio::fs::try_exists(temp_dir.path().join("key2")).await.unwrap());
		assert!(tokio::fs::try_exists(temp_dir.path().join("key2").with_extension(ENTRY_METADATA_EXTENSION)).await.unwrap());
		assert!(tokio::fs::try_exists(temp_dir.path().join("key3")).await.unwrap());
		assert!(tokio::fs::try_exists(temp_dir.path().join("key3").with_extension(ENTRY_METADATA_EXTENSION)).await.unwrap());
		
		let now = OffsetDateTime::now_utc();
		let query = artifact_cache.get_or_generate(5).await.unwrap();
		
		assert_eq!(query.entry_data, "stuff5");
		assert_eq!(query.metadata, "meta5");
		assert!(query.creation_date > now);
		
		{
			let lru_state = artifact_cache.entry_tracker.get_mut().unwrap();
			assert_eq!(lru_state.entries.iter().map(|e| e.0.as_str()).collect::<Vec<&str>>(), &["key1", "key44", "key5"]);
			assert_eq!(lru_state.total_size, 19);
			assert_eq!(lru_state.size_limit, 20);
		}
		
		assert!(!tokio::fs::try_exists(temp_dir.path().join("key2")).await.unwrap());
		assert!(!tokio::fs::try_exists(temp_dir.path().join("key2").with_extension(ENTRY_METADATA_EXTENSION)).await.unwrap());
		assert!(!tokio::fs::try_exists(temp_dir.path().join("key3")).await.unwrap());
		assert!(!tokio::fs::try_exists(temp_dir.path().join("key3").with_extension(ENTRY_METADATA_EXTENSION)).await.unwrap());
	}
}
