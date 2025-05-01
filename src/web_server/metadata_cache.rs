use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

pub struct FileMetadataCache {
	cache_tables: Mutex<HashMap<TypeId, Box<dyn Any + Sync + Send>>>,
}

struct MetadataEntry<T> {
	file_size: u64,
	last_modified: Option<SystemTime>,
	metadata: T,
}

pub trait FileMetadata: Sized + Clone {
	async fn fetch_metadata(path: &Path, file_metadata: &std::fs::Metadata) -> anyhow::Result<Self>;
}

impl<T> MetadataEntry<T> {
	fn still_valid(&self, file_metadata: &std::fs::Metadata) -> bool {
		self.file_size == file_metadata.len() &&
			self.last_modified == file_metadata.modified().ok()
	}
}

impl FileMetadataCache {
	pub fn new() -> Self {
		Self {
			cache_tables: Mutex::new(HashMap::new()),
		}
	}
	
	pub async fn fetch_metadata<T>(&self, path: impl AsRef<Path>) -> anyhow::Result<T>
	where
		T: FileMetadata + Sync + Send + 'static
	{
		let file_metadata = tokio::fs::metadata(&path).await?;
		
		self.fetch_metadata_with_meta(path, &file_metadata).await
	}
	
	pub async fn fetch_metadata_with_meta<T>(&self, path: impl AsRef<Path>, file_metadata: &std::fs::Metadata) -> anyhow::Result<T>
	where
		T: FileMetadata + Sync + Send + 'static
	{
		let media_path = path.as_ref();
		
		{
			let mut cache_tables = self.cache_tables.lock().unwrap();
			
			if let Some(entry) = Self::get_table::<T>(&mut *cache_tables).get(media_path) {
				if entry.still_valid(file_metadata) {
					return Ok(entry.metadata.clone());
				}
			}
		}
		
		let media_metadata = T::fetch_metadata(media_path, file_metadata).await?;
		
		{
			let mut cache_tables = self.cache_tables.lock().unwrap();
			
			Self::get_table::<T>(&mut *cache_tables).insert(media_path.to_owned(), MetadataEntry {
				file_size: file_metadata.len(),
				last_modified: file_metadata.modified().ok(),
				metadata: media_metadata.clone(),
			});
		}
		
		Ok(media_metadata)
	}
	
	fn get_table<T>(cache_tables: &mut HashMap<TypeId, Box<dyn Any + Sync + Send>>) -> &mut HashMap<PathBuf, MetadataEntry<T>>
	where
		T: FileMetadata + Sync + Send + 'static
	{
		cache_tables.entry(TypeId::of::<T>())
			.or_insert_with(|| Box::new(HashMap::<PathBuf, MetadataEntry<T>>::new()))
			.downcast_mut()
			.expect("")
	}
}