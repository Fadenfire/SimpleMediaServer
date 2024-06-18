use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use http::{Method, StatusCode};
use serde::Serialize;
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::libraries::reconstruct_library_path;
use crate::web_server::router::ServerState;
use crate::web_server::video_locator;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, json_response, restrict_method};

#[instrument(skip(server_state, request))]
pub async fn list_dir_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let resolved_path = server_state.libraries.resolve_path(library_id, library_path)?;
	
	let file_metadata = tokio::fs::metadata(&resolved_path).await?;
	
	if !file_metadata.is_dir() {
		return Err(ApiError::NotADirectory);
	}
	
	let mut read_dir = tokio::fs::read_dir(&resolved_path).await?;
	
	let mut files: Vec<FileEntry> = Vec::new();
	let mut directories: Vec<DirectoryEntry> = Vec::new();
	
	let mut total_time = Duration::from_millis(0);
	
	let mut file_stem_set: HashSet<String> = HashSet::new();
	
	while let Some(entry) = read_dir.next_entry().await? {
		let path = entry.path();
		let file_type = entry.file_type().await?;
		
		if file_type.is_file() {
			if !video_locator::is_video(&path) { continue; }
			
			let Some(path_name) = path.file_stem().and_then(OsStr::to_str) else { continue };
			
			if file_stem_set.contains(path_name) { continue; }
			file_stem_set.insert(path_name.to_owned());
			
			let media_metadata = server_state.video_metadata_cache.fetch_media_metadata(&path, &server_state.thumbnail_sheet_generator).await?;
			let thumbnail_path = format!("/api/thumbnail/{}/{}", reconstruct_library_path(library_id, library_path), path_name);
			
			total_time += media_metadata.duration;
			
			files.push(FileEntry {
				path_name: path_name.to_owned(),
				display_name: media_metadata.title,
				thumbnail_path,
				duration: media_metadata.duration.as_secs(),
			});
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
						format!("/api/thumbnail/{}/{}/{}", reconstruct_library_path(library_id, library_path), path_name, thumbnail_path_name)
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
	
	directories.sort_by(|a, b| natord::compare(&a.path_name, &b.path_name));
	files.sort_by(|a, b| natord::compare(&a.path_name, &b.path_name));
	
	let res = ListDirResponse {
		files,
		directories,
		total_duration: total_time.as_secs(),
	};
	
	Ok(json_response(StatusCode::OK, &res))
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
pub struct ListDirResponse {
	files: Vec<FileEntry>,
	directories: Vec<DirectoryEntry>,
	total_duration: u64,
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