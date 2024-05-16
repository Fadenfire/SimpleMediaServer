use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::Metadata;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{extract, Json};
use axum::extract::State;
use serde::Serialize;
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::state::ServerState;
use crate::web_server::video_locator;
use crate::web_server::video_locator::MEDIA_EXTENSIONS;

#[instrument(skip(server_state))]
pub async fn file_info_route(
	State(server_state): State<Arc<ServerState>>,
	extract::Path(library_path): extract::Path<String>,
) -> Result<Json<FileInfoResponse>, ApiError> {
	let (library, rel_path) = server_state.libraries.split_library_path(&library_path)?;
	let resolved_path = library.resolve_path(rel_path).ok_or(ApiError::FileNotFound)?;
	let resolved_path = video_locator::locate_video(&resolved_path).await.map_err(|_| ApiError::FileNotFound)?;
	
	let file_metadata = tokio::fs::metadata(&resolved_path).await.map_err(ApiError::from_io_error)?;
	
	if file_metadata.is_file() {
		file_info(server_state, file_metadata, resolved_path).await.map_err(ApiError::from_unknown_err)
	} else if file_metadata.is_dir() {
		let display_name = if rel_path.is_empty() {
			library.display_name.clone()
		} else {
			resolved_path.file_name()
				.map(OsStr::to_string_lossy)
				.map(Cow::into_owned)
				.ok_or(ApiError::FileNotFound)?
		};

		directory_info(server_state, resolved_path, display_name).await.map_err(ApiError::from_unknown_err)
	} else {
		Err(ApiError::FileNotFound)
	}
}

async fn file_info(server_state: Arc<ServerState>, file_metadata: Metadata, path: PathBuf) -> anyhow::Result<Json<FileInfoResponse>> {
	let video_metadata = server_state.video_metadata_cache.fetch_video_metadata(&path).await?;

	let file_info = FileInfo {
		display_name: video_metadata.title,
		size: file_metadata.len(),
		duration: video_metadata.duration.as_secs(),
		artist: video_metadata.artist,
	};

	Ok(Json(FileInfoResponse::File(file_info)))
}

async fn directory_info(server_state: Arc<ServerState>, dir_path: PathBuf, display_name: String) -> anyhow::Result<Json<FileInfoResponse>> {
	let mut read_dir = tokio::fs::read_dir(&dir_path).await?;

	let mut files: Vec<ChildFile> = Vec::new();
	let mut directories: Vec<ChildDirectory> = Vec::new();

	while let Some(entry) = read_dir.next_entry().await? {
		let path = entry.path();
		let file_type = entry.file_type().await?;

		if file_type.is_file() {
			let is_video = path
				.extension()
				.and_then(OsStr::to_str)
				.filter(|ext| MEDIA_EXTENSIONS.contains(ext))
				.is_some();

			if !is_video { continue; }
			
			let name = match path.file_stem().and_then(OsStr::to_str) {
				Some(s) => s,
				None => continue
			};

			let video_metadata = server_state.video_metadata_cache.fetch_video_metadata(&path).await?;

			files.push(ChildFile {
				path_name: name.to_owned(),
				display_name: video_metadata.title,
				duration: video_metadata.duration.as_secs(),
			});
		} else if file_type.is_dir() {
			let name = match path.file_name().and_then(OsStr::to_str) {
				Some(s) => s,
				None => continue
			};
			
			directories.push(ChildDirectory {
				path_name: name.to_owned(),
			});
		}
	}

	directories.sort_by_key(|it| it.path_name.clone());
	files.sort_by_key(|it| it.path_name.clone());

	let dir_info = DirectoryInfo {
		display_name,
		files,
		directories,
	};
	
	Ok(Json(FileInfoResponse::Directory(dir_info)))
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
}

#[derive(Debug, Serialize)]
pub struct DirectoryInfo {
	display_name: String,
	files: Vec<ChildFile>,
	directories: Vec<ChildDirectory>,
}

#[derive(Debug, Serialize)]
pub struct ChildFile {
	path_name: String,
	display_name: String,
	duration: u64,
}

#[derive(Debug, Serialize)]
pub struct ChildDirectory {
	path_name: String,
}