use std::sync::Arc;

use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use serde::Serialize;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::state::ServerState;

mod list_libraries;
pub mod error;
mod file_info;
mod thumbnail;

pub fn build_router() -> Router<Arc<ServerState>> {
	Router::new()
		.route("/libraries", get(list_libraries::list_libraries_route))
		.route("/file_info/*library_path", get(file_info::file_info_route))
		.route("/thumbnail/*library_path", get(thumbnail::thumbnail_route))
		.fallback(|| async { ApiError::NotFound.into_response() })
}
