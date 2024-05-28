use std::borrow::Cow;
use std::ffi::OsStr;
use std::sync::Arc;

use anyhow::Context;
use axum::{extract, Json};
use axum::extract::State;
use serde::Serialize;
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::api_routes::list_dir;
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
		let media_metadata = server_state.video_metadata_cache.fetch_media_metadata(&resolved_path, &server_state.thumbnail_sheet_generator).await?;
		
		let adjacent_files = list_dir::collect_video_list(&resolved_path.parent().context("No parent")?).await?;
		let this_index = adjacent_files.iter().position(|path| path == &resolved_path).context("Can't find self in file list")?;
		
		let video_info = media_metadata.video_metadata.map(|video_metadata| {
			VideoInfo {
				video_size: video_metadata.video_size,
				sheet_thumbnail_size: Dimension {
					width: video_metadata.thumbnail_sheet_params.thumbnail_width,
					height: video_metadata.thumbnail_sheet_params.thumbnail_height,
				},
				thumbnail_sheet_rows: video_metadata.thumbnail_sheet_params.sheet_rows,
				thumbnail_sheet_cols: video_metadata.thumbnail_sheet_params.sheet_cols,
				thumbnail_sheet_interval: video_metadata.thumbnail_sheet_params.interval,
			}
		});
		
		let file_info = MediaInfo {
			path: library_path.clone(),
			display_name: media_metadata.title,
			file_size: file_metadata.len(),
			duration: media_metadata.duration.as_secs(),
			artist: media_metadata.artist,
			video_info,
			prev_video: this_index.checked_sub(1).and_then(|i| adjacent_files.get(i)).and_then(|path| path.file_stem()).and_then(OsStr::to_str).map(ToOwned::to_owned),
			next_video: adjacent_files.get(this_index + 1).and_then(|path| path.file_stem()).and_then(OsStr::to_str).map(ToOwned::to_owned),
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
	File(MediaInfo),
	Directory(DirectoryInfo),
}

#[derive(Debug, Serialize)]
pub struct MediaInfo {
	path: String,
	display_name: String,
	file_size: u64,
	duration: u64,
	artist: Option<String>,
	video_info: Option<VideoInfo>,
	prev_video: Option<String>,
	next_video: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VideoInfo {
	video_size: Dimension,
	sheet_thumbnail_size: Dimension,
	thumbnail_sheet_rows: u32,
	thumbnail_sheet_cols: u32,
	thumbnail_sheet_interval: u32,
}

#[derive(Debug, Serialize)]
pub struct DirectoryInfo {
	display_name: String,
}
