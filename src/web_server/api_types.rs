use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::web_server::media_metadata::Dimension;

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
	pub file_size: u64,
	pub artist: Option<String>,
	pub watch_progress: Option<u64>,
	#[serde(with = "time::serde::iso8601")]
	pub creation_date: OffsetDateTime,
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
	pub library_display_name: String,
	pub display_name: String,
	pub file_size: u64,
	pub duration: u64,
	pub artist: Option<String>,
	#[serde(with = "time::serde::iso8601")]
	pub creation_date: OffsetDateTime,
	pub thumbnail_path: String,
	pub video_info: Option<ApiVideoInfo>,
	pub subtitle_streams: Vec<ApiSubtitleStream>,
	pub prev_video: Option<String>,
	pub next_video: Option<String>,
	pub watch_progress: Option<u64>,
	pub description: Option<String>,
	pub connections: Vec<ApiVideoConnection>,
	pub comments: Vec<ApiCommentThread>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiSubtitleStream {
	pub index: usize,
	pub language: Option<String>,
	pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiCommentThread {
	pub comment: ApiComment,
	pub replies: Vec<ApiComment>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiComment {
	pub author: String,
	pub text: String,
	pub likes: u64,
	#[serde(with = "time::serde::iso8601")]
	pub published_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct ApiDirectoryInfo {
	pub full_path: RelativePathBuf,
	pub library_display_name: String,
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
