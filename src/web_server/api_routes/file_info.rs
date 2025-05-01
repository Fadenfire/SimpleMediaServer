use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::Path;

use anyhow::Context;
use http::{Method, StatusCode};
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use crate::media_manipulation::thumbnail_sheet;
use crate::web_server::{libraries, media_connections, video_locator};
use crate::web_server::api_error::ApiError;
use crate::web_server::api_routes::{list_dir, thumbnail};
use crate::web_server::api_types::{ApiCommentThread, ApiDirectoryInfo, ApiFileInfo, ApiVideoConnection, ApiVideoInfo};
use crate::web_server::auth::User;
use crate::web_server::libraries::Library;
use crate::web_server::server_state::ServerState;
use crate::web_server::video_locator::LocatedFile;
use crate::web_server::media_metadata::{AdvancedMediaMetadata, BasicMediaMetadata, Dimension};
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
	
	let res = match video_locator::locate_video(&resolved_path).await? {
		LocatedFile::File(file_path) => {
			let file_info = create_file_info(server_state, user, library, &library_path, &file_path).await?;
			
			FileInfoResponse::File(file_info)
		}
		LocatedFile::Directory(dir_path) => {
			let display_name = if library_path.as_str().is_empty() {
				library.display_name.clone()
			} else {
				dir_path.file_name()
					.map(OsStr::to_string_lossy)
					.map(Cow::into_owned)
					.ok_or(ApiError::FileNotFound)?
			};
			
			let dir_info = ApiDirectoryInfo {
				full_path: RelativePath::new(&library.id).join(&library_path),
				library_display_name: library.display_name.clone(),
				display_name,
			};
			
			FileInfoResponse::Directory(dir_info)
		}
	};
	
	Ok(json_response(StatusCode::OK, &res))
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum FileInfoResponse {
	File(ApiFileInfo),
	Directory(ApiDirectoryInfo),
}

pub const DESCRIPTION_FILE_EXT: &str = "description";
pub const COMMENTS_FILE_EXT: &str = "comments.json";

pub async fn create_file_info(
	server_state: &ServerState,
	user: &User,
	library: &Library,
	library_path: &RelativePath,
	media_path: &Path
) -> anyhow::Result<ApiFileInfo> {
	let file_metadata = tokio::fs::metadata(&media_path).await?;
	
	let basic_metadata = server_state.metadata_cache
		.fetch_metadata_with_meta::<BasicMediaMetadata>(&media_path, &file_metadata).await?;
	let advanced_metadata = server_state.metadata_cache
		.fetch_metadata_with_meta::<AdvancedMediaMetadata>(&media_path, &file_metadata).await?;
	
	let adjacent_files = list_dir::collect_video_list(&media_path.parent().context("No parent")?).await?;
	let this_index = adjacent_files.iter().position(|path| path == &media_path).context("Can't find self in file list")?;
	
	let video_info = match &advanced_metadata.video_metadata {
		Some(video_metadata) => {
			let thumbnail_sheet_params = server_state.thumbnail_sheet_generator.get(&media_path.to_owned()).await?
				.map(|entry| entry.metadata)
				.unwrap_or_else(|| thumbnail_sheet::calculate_sheet_params(
					advanced_metadata.ffmpeg_duration,
					video_metadata.video_size.width,
					video_metadata.video_size.height
				));
			
			Some(ApiVideoInfo {
				video_size: video_metadata.video_size.clone(),
				sheet_thumbnail_size: Dimension {
					width: thumbnail_sheet_params.thumbnail_width,
					height: thumbnail_sheet_params.thumbnail_height,
				},
				thumbnail_sheet_rows: thumbnail_sheet_params.sheet_rows,
				thumbnail_sheet_cols: thumbnail_sheet_params.sheet_cols,
				thumbnail_sheet_interval: thumbnail_sheet_params.interval,
			})
		},
		None => None,
	};
	
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
		.get_entry(&library.id, &library_path)
		.map(|entry| entry.progress);
	
	let description = read_file_maybe(media_path, DESCRIPTION_FILE_EXT).await?
		.and_then(|data| String::from_utf8(data).ok());
	
	let mut connections: Vec<ApiVideoConnection> = media_connections::get_video_connections(&media_path, library_path, library).await?
		.into_iter()
		.flat_map(|entry| {
			let other_path = RelativePath::new(&library.id).join(&entry.video_path);
			let other_thumbnail = thumbnail::create_thumbnail_path(&other_path);
			let shortcut_thumbnail = entry.shortcut_thumbnail
				.map(|path| thumbnail::create_thumbnail_path(&RelativePath::new(&library.id).join(&path)));
			
			entry.connections.into_iter()
				.map(move |connection| ApiVideoConnection {
					video_path: other_path.clone(),
					video_thumbnail: other_thumbnail.clone(),
					relation: entry.relation.clone(),
					shortcut_thumbnail: shortcut_thumbnail.clone(),
					left_start: connection.left_start,
					left_end: connection.left_start + connection.duration,
					right_start: connection.right_start,
				})
		})
		.collect();
	
	connections.sort_by_key(|con| con.left_end);
	
	let comments = read_file_maybe(media_path, COMMENTS_FILE_EXT).await?
		.and_then(|data| serde_json::from_slice(&data).ok())
		.map(|comments_file: CommentsFile| comments_file.comment_threads)
		.unwrap_or_default();
	
	Ok(ApiFileInfo {
		full_path: RelativePath::new(&library.id).join(&library_path),
		library_display_name: library.display_name.clone(),
		display_name: basic_metadata.title,
		file_size: file_metadata.len(),
		duration: basic_metadata.duration.as_secs(),
		artist: basic_metadata.artist,
		creation_date: basic_metadata.creation_date,
		video_info,
		prev_video,
		next_video,
		watch_progress,
		description,
		connections,
		comments,
	})
}

pub async fn read_file_maybe(media_path: &Path, ext: &str) -> anyhow::Result<Option<Vec<u8>>> {
	let path = media_path.with_extension(ext);
	
	if !tokio::fs::try_exists(&path).await? {
		return Ok(None);
	}
	
	Ok(Some(tokio::fs::read(&path).await?))
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommentsFile {
	pub comment_threads: Vec<ApiCommentThread>,
}
