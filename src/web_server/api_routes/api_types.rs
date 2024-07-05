use relative_path::RelativePathBuf;
use serde::Serialize;
use time::OffsetDateTime;

use crate::web_server::video_metadata::Dimension;

#[derive(Debug, Serialize)]
pub struct ApiUserInfo {
	pub display_name: String,
	pub username: String,
}

#[derive(Debug, Serialize)]
pub struct ApiLibraryEntry {
	pub id: String,
	pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct ApiFileEntry {
	pub path_name: String,
	pub full_path: RelativePathBuf,
	pub display_name: String,
	pub thumbnail_path: String,
	pub duration: u64,
	pub artist: Option<String>,
	pub watch_progress: Option<u64>,
	#[serde(with = "time::serde::iso8601")]
	pub date_modified: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct ApiDirectoryEntry {
	pub path_name: String,
	pub display_name: String,
	pub thumbnail_path: Option<String>,
	pub child_count: u32,
}

#[derive(Debug, Serialize)]
pub struct ApiFileInfo {
	pub full_path: RelativePathBuf,
	pub display_name: String,
	pub file_size: u64,
	pub duration: u64,
	pub artist: Option<String>,
	pub video_info: Option<ApiVideoInfo>,
	pub prev_video: Option<String>,
	pub next_video: Option<String>,
	pub watch_progress: Option<u64>,
	pub connections: Vec<ApiVideoConnection>,
}

#[derive(Debug, Serialize)]
pub struct ApiVideoInfo {
	pub video_size: Dimension,
	pub sheet_thumbnail_size: Dimension,
	pub thumbnail_sheet_rows: u32,
	pub thumbnail_sheet_cols: u32,
	pub thumbnail_sheet_interval: u32,
}

#[derive(Debug, Serialize)]
pub struct ApiVideoConnection {
	pub video_path: RelativePathBuf,
	pub video_thumbnail: String,
	pub relation: String,
	pub shortcut_thumbnail: Option<String>,
	pub left_start: u64,
	pub left_end: u64,
	pub right_start: u64,
}

#[derive(Debug, Serialize)]
pub struct ApiDirectoryInfo {
	pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct ApiWatchHistoryEntry {
	pub library_id: String,
	pub media_path: RelativePathBuf,
	#[serde(with = "time::serde::iso8601")]
	pub last_watched: OffsetDateTime,
	pub progress: u64,
	pub file: Option<ApiFileEntry>,
}
