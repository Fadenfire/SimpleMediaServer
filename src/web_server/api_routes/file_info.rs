use std::borrow::Cow;
use std::ffi::OsStr;

use anyhow::Context;
use http::{Method, StatusCode};
use relative_path::{RelativePath, RelativePathBuf};
use serde::Serialize;
use tracing::instrument;

use crate::web_server::{libraries, video_locator};
use crate::web_server::api_routes::error::ApiError;
use crate::web_server::api_routes::list_dir;
use crate::web_server::server_state::ServerState;
use crate::web_server::video_metadata::Dimension;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, json_response, restrict_method};

#[instrument(skip(server_state, request))]
pub async fn file_info_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str]
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let user = server_state.auth_manager.lookup_from_headers(request.headers())?;
	
	let library_path: RelativePathBuf = library_path.iter().collect();
	let (library, resolved_path) = libraries::resolve_library_and_path_with_auth(
		server_state, library_id, library_path.clone(), request.headers())?;
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
		
		let prev_video = this_index
			.checked_sub(1)
			.and_then(|i| adjacent_files.get(i))
			.and_then(|path| path.file_stem())
			.and_then(OsStr::to_str)
			.map(ToOwned::to_owned);
		let next_video = adjacent_files.get(this_index + 1)
			.and_then(|path| path.file_stem())
			.and_then(OsStr::to_str)
			.map(ToOwned::to_owned);
		
		let watch_progress = server_state.user_watch_histories.lock().unwrap()
			.get_watch_history(&user.id)
			.get_entry(library_id, &library_path)
			.map(|entry| entry.progress);
		
		let file_info = MediaInfo {
			path: RelativePath::new(library_id).join(&library_path),
			display_name: media_metadata.title,
			file_size: file_metadata.len(),
			duration: media_metadata.duration.as_secs(),
			artist: media_metadata.artist,
			video_info,
			prev_video,
			next_video,
			watch_progress,
		};
		
		Ok(json_response(StatusCode::OK, &FileInfoResponse::File(file_info)))
	} else if file_metadata.is_dir() {
		let display_name = if library_path.as_str().is_empty() {
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
		
		Ok(json_response(StatusCode::OK, &FileInfoResponse::Directory(dir_info)))
	} else {
		Err(ApiError::FileNotFound)
	}
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum FileInfoResponse {
	File(MediaInfo),
	Directory(DirectoryInfo),
}

#[derive(Debug, Serialize)]
struct MediaInfo {
	path: RelativePathBuf,
	display_name: String,
	file_size: u64,
	duration: u64,
	artist: Option<String>,
	video_info: Option<VideoInfo>,
	prev_video: Option<String>,
	next_video: Option<String>,
	watch_progress: Option<u64>,
}

#[derive(Debug, Serialize)]
struct VideoInfo {
	video_size: Dimension,
	sheet_thumbnail_size: Dimension,
	thumbnail_sheet_rows: u32,
	thumbnail_sheet_cols: u32,
	thumbnail_sheet_interval: u32,
}

#[derive(Debug, Serialize)]
struct DirectoryInfo {
	display_name: String,
}
