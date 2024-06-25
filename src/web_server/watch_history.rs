use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::future::join_all;
use hashlink::linked_hash_map::Entry;
use hashlink::LinkedHashMap;
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::Notify;

use crate::web_server::auth::AuthManager;

pub struct UserWatchHistories {
	watch_histories: HashMap<String, WatchHistory>,
	dirty_notify: Arc<Notify>,
}

impl UserWatchHistories {
	pub async fn load(users: &AuthManager, watch_histories_dir: PathBuf) -> anyhow::Result<Arc<Mutex<Self>>> {
		tokio::fs::create_dir_all(&watch_histories_dir).await?;
		
		let mut watch_histories = HashMap::new();
		
		for user in users.iter_users() {
			let history_file = watch_histories_dir.join(format!("{}.json", user.id));
			
			watch_histories.insert(user.id.clone(), WatchHistory::load(history_file).await?);
		}
		
		let dirty_notify = Arc::new(Notify::new());
		
		let arc_self = Arc::new(Mutex::new(Self {
			watch_histories,
			dirty_notify: dirty_notify.clone(),
		}));
		
		tokio::spawn(Self::save_task(arc_self.clone(), dirty_notify, watch_histories_dir));
		
		Ok(arc_self)
	}
	
	async fn save_task(arc_self: Arc<Mutex<Self>>, dirty_notify: Arc<Notify>, watch_histories_dir: PathBuf) {
		loop {
			dirty_notify.notified().await;
			
			let mut futures = Vec::new();
			
			{
				let mut_self = arc_self.lock().unwrap();
				
				for (user_id, watch_history) in mut_self.watch_histories.iter() {
					if watch_history.dirty {
						let history_file = watch_histories_dir.join(format!("{}.json", user_id));
						let ser_watch_history = watch_history.serialize();
						let data = serde_json::to_vec_pretty(&ser_watch_history).unwrap();
						
						futures.push(tokio::fs::write(history_file, data));
					}
				}
			}
			
			join_all(futures).await;
			
			tokio::time::sleep(Duration::from_secs(60)).await;
		}
	}
	
	pub fn get_watch_history(&mut self, user_id: &str) -> &mut WatchHistory {
		self.watch_histories.get_mut(user_id).expect("Unknown user id")
	}
	
	pub fn mark_dirty(&self) {
		self.dirty_notify.notify_one();
	}
}

pub struct WatchHistory {
	entries: LinkedHashMap<MediaKey, WatchHistoryEntry>,
	dirty: bool,
}

impl WatchHistory {
	pub async fn load(history_file: PathBuf) -> anyhow::Result<Self> {
		let entries = if tokio::fs::try_exists(&history_file).await? {
			let data = tokio::fs::read(&history_file).await?;
			let watch_history: SerializedWatchHistory = serde_json::from_slice(&data)?;
			
			watch_history.entries
		} else {
			Vec::new()
		};
		
		Ok(Self::new(entries))
	}
	
	pub fn new(mut ser_entries: Vec<SerializedWatchHistoryEntry>) -> Self {
		let mut entries = LinkedHashMap::new();
		
		ser_entries.sort_by_key(|entry| entry.last_watched);
		
		for ser_entry in ser_entries {
			let key = MediaKey {
				library_id: ser_entry.library_id.clone(),
				media_path: ser_entry.media_path.clone(),
			};
			
			let entry = WatchHistoryEntry {
				library_id: ser_entry.library_id,
				media_path: ser_entry.media_path,
				last_watched: ser_entry.last_watched,
				progress: ser_entry.progress,
			};
			
			entries.insert(key, entry);
		}
		
		Self {
			entries,
			dirty: false,
		}
	}
	
	pub fn iter_entries(&self) -> impl Iterator<Item = &WatchHistoryEntry> {
		self.entries.values()
	}
	
	pub fn get_entry(&self, library_id: &str, media_path: &RelativePath) -> Option<&WatchHistoryEntry> {
		self.entries.get(&MediaKey::new(library_id, media_path.normalize()))
	}
	
	pub fn update_progress(&mut self, library_id: &str, media_path: &RelativePath, new_progress: u64) {
		let media_path = media_path.normalize();
		let updated_time = OffsetDateTime::now_utc();
		
		match self.entries.entry(MediaKey::new(library_id, media_path.clone())) {
			Entry::Occupied(mut entry) => {
				let key = entry.key().clone();
				
				let entry = entry.get_mut();
				entry.last_watched = updated_time;
				entry.progress = new_progress;
				
				self.entries.to_back(&key);
			}
			Entry::Vacant(entry) => {
				entry.insert(WatchHistoryEntry {
					library_id: library_id.to_owned(),
					media_path,
					last_watched: updated_time,
					progress: new_progress,
				});
			}
		}
		
		self.dirty = true;
	}
	
	fn serialize(&self) -> SerializedWatchHistory {
		SerializedWatchHistory {
			version: 1,
			entries: self.entries.values()
				.map(|entry| SerializedWatchHistoryEntry {
					library_id: entry.library_id.clone(),
					media_path: entry.media_path.clone(),
					last_watched: entry.last_watched,
					progress: entry.progress,
				})
				.collect(),
		}
	}
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct MediaKey {
	library_id: String,
	media_path: RelativePathBuf,
}

impl MediaKey {
	fn new(library_id: &str, media_path: RelativePathBuf) -> Self {
		Self {
			library_id: library_id.to_owned(),
			media_path,
		}
	}
}

#[derive(Debug)]
pub struct WatchHistoryEntry {
	pub library_id: String,
	pub media_path: RelativePathBuf,
	pub last_watched: OffsetDateTime,
	pub progress: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SerializedWatchHistory {
	version: u32,
	entries: Vec<SerializedWatchHistoryEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SerializedWatchHistoryEntry {
	library_id: String,
	media_path: RelativePathBuf,
	last_watched: OffsetDateTime,
	progress: u64,
}