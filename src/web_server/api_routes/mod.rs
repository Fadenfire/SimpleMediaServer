use std::sync::Arc;

use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::state::ServerState;

pub mod error;
mod list_libraries;
mod file_info;
mod thumbnail;
mod native_video;
mod list_dir;
mod thumbnail_sheet;

pub fn build_router() -> Router<Arc<ServerState>> {
	Router::new()
		.route("/libraries", get(list_libraries::list_libraries_route))
		.route("/file_info/*library_path", get(file_info::file_info_route))
		.route("/list_dir/*library_path", get(list_dir::list_dir_route))
		.route("/thumbnail/*library_path", get(thumbnail::thumbnail_route))
		.route("/thumbnail_sheet/*library_path", get(thumbnail_sheet::thumbnail_sheet_route))
		.route("/media/source/*library_path", get(native_video::native_video_route))
		.fallback(|| async { ApiError::NotFound.into_response() })
}
