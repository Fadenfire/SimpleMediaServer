use std::sync::Arc;

use crate::web_server::api_error::ApiError;
use crate::web_server::api_routes::login::login_route;
use crate::web_server::server_state::ServerState;
use crate::web_server::web_utils::{HyperRequest, HyperResponse};

mod list_libraries;
mod file_info;
mod list_dir;
mod thumbnail;
mod thumbnail_sheet;
mod native_video;
mod hls_manifest;
mod hls_segment;
mod login;
mod get_user;
mod update_watch_progress;
mod get_watch_history;
mod delete_watch_progress;
mod hls_level_manifest;

pub async fn route_request(request: HyperRequest, path: &[&str], server_state: Arc<ServerState>) -> HyperResponse {
	if let ["login"] = path {
		return login_route(request, &server_state.auth_manager).await.unwrap_or_else(ApiError::into_response);
	}
	
	if server_state.auth_manager.lookup_from_headers(request.headers()).is_err() {
		return ApiError::Unauthorized.into_response();
	}
	
	let result = match path {
		["get_user"] => get_user::get_user_route(&request, &server_state.auth_manager).await,
		["libraries"] => list_libraries::list_libraries_route(&server_state, &request).await,
		["update_watch_progress"] => update_watch_progress::update_watch_progress_route(&server_state, request).await,
		["delete_watch_progress"] => delete_watch_progress::delete_watch_progress_route(&server_state, request).await,
		["watch_history"] => get_watch_history::get_watch_history_route(&server_state, &request).await,
		
		["file_info", library_id, library_path @ ..] =>
			file_info::file_info_route(&server_state, &request, library_id, library_path).await,
		
		["list_dir", library_id, library_path @ ..] =>
			list_dir::list_dir_route(&server_state, &request, library_id, library_path).await,
		
		["thumbnail", library_id, library_path @ ..] =>
			thumbnail::thumbnail_route(&server_state, &request, library_id, library_path).await,
		
		["thumbnail_sheet", library_id, library_path @ ..] =>
			thumbnail_sheet::thumbnail_sheet_route(&server_state, &request, library_id, library_path).await,
		
		["media", "native", library_id, library_path @ ..] =>
			native_video::native_video_route(&server_state, request, library_id, library_path).await,
		
		["media", "hls", library_id, library_path @ .., "level", quality_level, "manifest.m3u8"] =>
			hls_level_manifest::hls_level_manifest_route(&server_state, &request, library_id, library_path, quality_level).await,
		
		["media", "hls", library_id, library_path @ .., "level", quality_level, "segment", segment_index] =>
			hls_segment::hls_segment_route(&server_state, &request, library_id, library_path, quality_level, segment_index).await,
		
		["media", "hls", library_id, library_path @ .., "manifest.m3u8"] =>
			hls_manifest::hls_manifest_route(&server_state, &request, library_id, library_path).await,
		
		_ => Err(ApiError::NotFound)
	};
	
	result.unwrap_or_else(ApiError::into_response)
}