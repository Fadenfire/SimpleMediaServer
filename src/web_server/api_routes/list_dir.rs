use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use http::{Method, StatusCode};
use relative_path::{RelativePath, RelativePathBuf};
use serde::Serialize;
use tracing::{error, instrument};

use crate::web_server::{libraries, video_locator};
use crate::web_server::api_error::ApiError;
use crate::web_server::api_routes::thumbnail;
use crate::web_server::api_types::{ApiDirectoryEntry, ApiFileEntry};
use crate::web_server::auth::User;
use crate::web_server::server_state::ServerState;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, json_response, restrict_method};

#[instrument(skip(server_state, request))]
pub async fn list_dir_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let user = server_state.auth_manager.lookup_from_headers(request.headers())?;
	
	let library_path: RelativePathBuf = library_path.iter().collect();
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.clone(), request.headers())?;
	
	let file_metadata = tokio::fs::metadata(&resolved_path).await?;
	
	if !file_metadata.is_dir() {
		return Err(ApiError::NotADirectory);
	}
	
	let mut read_dir = tokio::fs::read_dir(&resolved_path).await?;
	
	let mut files: Vec<ApiFileEntry> = Vec::new();
	let mut directories: Vec<ApiDirectoryEntry> = Vec::new();
	
	let mut total_time = 0;
	
	let mut file_stem_set: HashSet<String> = HashSet::new();
	
	while let Some(entry) = read_dir.next_entry().await? {
		let path = entry.path();
		
		if !server_state.config.main_config.show_hidden_files &&
			path.file_name().and_then(OsStr::to_str).is_some_and(video_locator::is_hidden) {
			continue;
		}
		
		let file_type = entry.file_type().await?;
		
		if file_type.is_file() {
			if !video_locator::is_video(&path) { continue; }
			
			let Some(path_name) = path.file_stem().and_then(OsStr::to_str) else { continue };
			
			if file_stem_set.contains(path_name) { continue; }
			file_stem_set.insert(path_name.to_owned());
			
			let file_library_path = library_path.join(&path_name);
			
			match create_file_entry(server_state, user, library_id, &file_library_path, &path).await {
				Ok(file_entry) => {
					total_time += file_entry.duration;
					
					files.push(file_entry);
				}
				Err(err) => error!("Error collecting file metadata for {:?}: {:?}", &path, err),
			}
		} else if file_type.is_dir() {
			let Some(path_name) = path.file_name().and_then(OsStr::to_str) else { continue };
			
			let mut child_count: u32 = 0;
			let mut thumbnail_path: Option<String> = None;
			
			if let Ok(video_paths) = collect_video_list(&path).await {
				child_count = video_paths.len() as u32;
				
				thumbnail_path = video_paths.first()
					.and_then(|path| path.file_stem())
					.and_then(OsStr::to_str)
					.map(|thumbnail_path_name| {
						thumbnail::create_thumbnail_path(&RelativePath::new(library_id).join(&library_path).join(path_name).join(thumbnail_path_name))
					});
			}
			
			directories.push(ApiDirectoryEntry {
				path_name: path_name.to_owned(),
				display_name: path_name.to_owned(),
				thumbnail_path,
				child_count,
			});
		}
	}
	
	directories.sort_by(|a, b| natord::compare(&a.path_name, &b.path_name));
	files.sort_by(|a, b| natord::compare(&a.path_name, &b.path_name));
	
	let res = ListDirResponse {
		files,
		directories,
		total_duration: total_time,
	};
	
	Ok(json_response(StatusCode::OK, &res))
}

pub async fn create_file_entry(
	server_state: &ServerState,
	user: &User,
	library_id: &str,
	library_path: &RelativePath,
	media_path: &Path
) -> anyhow::Result<ApiFileEntry> {
	let media_metadata = server_state.video_metadata_cache.fetch_media_metadata(media_path,
		&server_state.thumbnail_sheet_generator).await?;
	
	let full_path = RelativePath::new(library_id).join(&library_path);
	let thumbnail_path = thumbnail::create_thumbnail_path(&full_path);
	
	let watch_progress = server_state.user_watch_histories.lock().unwrap()
		.get_watch_history(&user.id)
		.get_entry(library_id, &library_path)
		.map(|entry| entry.progress);
	
	Ok(ApiFileEntry {
		path_name: media_metadata.path_name,
		full_path,
		display_name: media_metadata.title,
		thumbnail_path,
		duration: media_metadata.duration.as_secs(),
		artist: media_metadata.artist,
		watch_progress,
		creation_date: media_metadata.creation_date,
	})
}

pub async fn collect_video_list(dir_path: &Path) -> anyhow::Result<Vec<PathBuf>> {
	let mut read_dir = tokio::fs::read_dir(dir_path).await?;
	let mut video_paths: Vec<PathBuf> = Vec::new();
	let mut file_stem_set: HashSet<String> = HashSet::new();
	
	while let Some(entry) = read_dir.next_entry().await? {
		let path = entry.path();
		
		if !entry.file_type().await?.is_file() || !video_locator::is_video(&path) { continue; }
		
		let Some(stem) = path.file_stem().and_then(OsStr::to_str) else { continue };
		
		if file_stem_set.contains(stem) { continue; }
		
		file_stem_set.insert(stem.to_owned());
		video_paths.push(path);
	}
	
	video_paths.sort_by(|a, b| natord::compare(
		a.file_stem().unwrap().to_str().unwrap(), // file_stem must have been Some to be added
		b.file_stem().unwrap().to_str().unwrap(), //  to the list, so this should be safe
	));
	
	Ok(video_paths)
}

#[derive(Debug, Serialize)]
struct ListDirResponse {
	files: Vec<ApiFileEntry>,
	directories: Vec<ApiDirectoryEntry>,
	total_duration: u64,
}
