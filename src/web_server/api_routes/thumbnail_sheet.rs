use std::sync::Arc;

use axum::extract;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::state::ServerState;
use crate::web_server::video_locator;
use crate::web_server::web_utils::serve_file_basic;

#[instrument(skip(server_state))]
pub async fn thumbnail_sheet_route(
	State(server_state): State<Arc<ServerState>>,
	extract::Path(library_path): extract::Path<String>,
	headers: HeaderMap,
) -> Result<Response, ApiError> {
	let resolved_path = server_state.libraries.resolve_path(&library_path)?;
	let media_path = video_locator::locate_video(&resolved_path).await.map_err(|_| ApiError::FileNotFound)?;
	
	let generated_spritesheet = server_state.thumbnail_sheet_generator.generate_image(media_path).await?;
	let res = serve_file_basic(&generated_spritesheet.cache_file, generated_spritesheet.mod_time, mime::IMAGE_JPEG, &headers).await?;
	
	Ok(res)
}