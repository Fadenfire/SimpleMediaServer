use std::sync::Arc;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::router::ServerState;
use crate::web_server::web_utils::{HyperRequest, HyperResponse};

pub mod error;
mod list_libraries;
mod file_info;
mod list_dir;
mod thumbnail;
mod thumbnail_sheet;
mod native_video;

pub async fn route_request(request: HyperRequest, path: &[&str], server_state: Arc<ServerState>) -> HyperResponse {
	let result = match path {
		["libraries"] => list_libraries::list_libraries_route(&request, &server_state.libraries).await,
		
		["file_info", library_id, library_path @ ..] =>
			file_info::file_info_route(&server_state, &request, *library_id, library_path).await,
		
		["list_dir", library_id, library_path @ ..] =>
			list_dir::list_dir_route(&server_state, &request, *library_id, library_path).await,
		
		["thumbnail", library_id, library_path @ ..] =>
			thumbnail::thumbnail_route(&server_state, &request, *library_id, library_path).await,
		
		["thumbnail_sheet", library_id, library_path @ ..] =>
			thumbnail_sheet::thumbnail_sheet_route(&server_state, &request, *library_id, library_path).await,
		
		["media", "source", library_id, library_path @ ..] =>
			native_video::native_video_route(&server_state, request, *library_id, library_path).await,
		
		_ => Err(ApiError::NotFound)
	};
	
	result.unwrap_or_else(ApiError::into_response)
}