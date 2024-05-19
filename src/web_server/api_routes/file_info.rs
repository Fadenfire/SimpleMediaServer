use std::borrow::Cow;
use std::ffi::OsStr;
use std::sync::Arc;

use axum::{extract, Json};
use axum::extract::State;
use serde::Serialize;
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::state::ServerState;
use crate::web_server::video_locator;
use crate::web_server::video_metadata::Dimension;

#[instrument(skip(server_state))]
pub async fn file_info_route(
	State(server_state): State<Arc<ServerState>>,
	extract::Path(library_path): extract::Path<String>,
) -> Result<Json<FileInfoResponse>, ApiError> {
	let (library, rel_path) = server_state.libraries.split_library_path(&library_path)?;
	let resolved_path = library.resolve_path(rel_path).ok_or(ApiError::FileNotFound)?;
	let resolved_path = video_locator::locate_video(&resolved_path).await.map_err(|_| ApiError::FileNotFound)?;
	
	let file_metadata = tokio::fs::metadata(&resolved_path).await?;
	
	if file_metadata.is_file() {
		let video_metadata = server_state.video_metadata_cache.fetch_video_metadata(&resolved_path).await?;
		
		let file_info = FileInfo {
			display_name: video_metadata.title,
			size: file_metadata.len(),
			duration: video_metadata.duration.as_secs(),
			artist: video_metadata.artist,
			video_resolution: video_metadata.video_resolution,
		};
		
		Ok(Json(FileInfoResponse::File(file_info)))
	} else if file_metadata.is_dir() {
		let display_name = if rel_path.is_empty() {
			library.display_name.clone()
		} else {
			resolved_path.file_name()
				.map(OsStr::to_string_lossy)
				.map(Cow::into_owned)
				.ok_or(ApiError::FileNotFound)?
		};
		
		let dir_info = DirectoryInfo {
			display_name,
		};
		
		Ok(Json(FileInfoResponse::Directory(dir_info)))
	} else {
		Err(ApiError::FileNotFound)
	}
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FileInfoResponse {
	File(FileInfo),
	Directory(DirectoryInfo),
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
	display_name: String,
	size: u64,
	duration: u64,
	artist: Option<String>,
	video_resolution: Option<Dimension>,
}

#[derive(Debug, Serialize)]
pub struct DirectoryInfo {
	display_name: String,
}
