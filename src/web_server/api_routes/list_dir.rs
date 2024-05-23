use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{extract, Json};
use axum::extract::State;
use serde::Serialize;
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::state::ServerState;
use crate::web_server::video_locator;

#[instrument(skip(server_state))]
pub async fn list_dir_route(
	State(server_state): State<Arc<ServerState>>,
	extract::Path(library_path): extract::Path<String>,
) -> Result<Json<ListDirResponse>, ApiError> {
	let resolved_path = server_state.libraries.resolve_path(&library_path)?;
	
	let file_metadata = tokio::fs::metadata(&resolved_path).await?;
	
	if !file_metadata.is_dir() {
		return Err(ApiError::NotADirectory);
	}
	
	let mut read_dir = tokio::fs::read_dir(&resolved_path).await?;
	
	let mut files: Vec<FileEntry> = Vec::new();
	let mut directories: Vec<DirectoryEntry> = Vec::new();
	
	while let Some(entry) = read_dir.next_entry().await? {
		let path = entry.path();
		let file_type = entry.file_type().await?;
		
		if file_type.is_file() {
			if !video_locator::is_video(&path) { continue; }
			
			let path_name = match path.file_stem().and_then(OsStr::to_str) {
				Some(s) => s,
				None => continue
			};
			
			let media_metadata = server_state.video_metadata_cache.fetch_media_metadata(&path).await?;
			let thumbnail_path = format!("/api/thumbnail/{}/{}", library_path.trim_end_matches('/'), path_name);
			
			files.push(FileEntry {
				path_name: path_name.to_owned(),
				display_name: media_metadata.title,
				thumbnail_path,
				duration: media_metadata.duration.as_secs(),
			});
		} else if file_type.is_dir() {
			let path_name = match path.file_name().and_then(OsStr::to_str) {
				Some(s) => s,
				None => continue
			};
			
			let mut child_count: u32 = 0;
			let mut thumbnail_path: Option<String> = None;
			
			if let Ok(video_paths) = collect_video_list(&path).await {
				child_count = video_paths.len() as u32;
				
				thumbnail_path = video_paths.first()
					.and_then(|path| path.file_stem())
					.and_then(|stem| stem.to_str())
					.map(|thumbnail_path_name| {
						format!("/api/thumbnail/{}/{}/{}", library_path.trim_end_matches('/'), path_name, thumbnail_path_name)
					});
			}
			
			directories.push(DirectoryEntry {
				path_name: path_name.to_owned(),
				display_name: path_name.to_owned(),
				thumbnail_path,
				child_count,
			});
		}
	}
	
	directories.sort_by_key(|it| it.path_name.clone());
	files.sort_by_key(|it| it.path_name.clone());
	
	let res = ListDirResponse {
		files,
		directories,
	};
	
	Ok(Json(res))
}

pub async fn collect_video_list(dir_path: &Path) -> anyhow::Result<Vec<PathBuf>> {
	let mut read_dir = tokio::fs::read_dir(dir_path).await?;
	let mut video_paths: Vec<PathBuf> = Vec::new();
	
	while let Some(entry) = read_dir.next_entry().await? {
		let path = entry.path();
		
		if entry.file_type().await?.is_file() && video_locator::is_video(&path) {
			video_paths.push(path);
		}
	}
	
	video_paths.sort_by_key(|path| path.file_name().map(ToOwned::to_owned));
	
	Ok(video_paths)
}

#[derive(Debug, Serialize)]
pub struct ListDirResponse {
	files: Vec<FileEntry>,
	directories: Vec<DirectoryEntry>,
}

#[derive(Debug, Serialize)]
pub struct FileEntry {
	path_name: String,
	display_name: String,
	thumbnail_path: String,
	duration: u64,
}

#[derive(Debug, Serialize)]
pub struct DirectoryEntry {
	path_name: String,
	display_name: String,
	thumbnail_path: Option<String>,
	child_count: u32,
}