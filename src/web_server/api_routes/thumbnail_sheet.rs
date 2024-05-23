use std::sync::Arc;

use axum::body::Body;
use axum::extract;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use headers::{HeaderMapExt, IfModifiedSince, LastModified};
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::state::ServerState;
use crate::web_server::video_locator;

#[instrument(skip(server_state))]
pub async fn thumbnail_sheet_route(
	State(server_state): State<Arc<ServerState>>,
	extract::Path(library_path): extract::Path<String>,
	headers: HeaderMap,
) -> Result<Response, ApiError> {
	let resolved_path = server_state.libraries.resolve_path(&library_path)?;
	let media_path = video_locator::locate_video(&resolved_path).await.map_err(|_| ApiError::FileNotFound)?;
	
	let if_modified_since: Option<IfModifiedSince> = headers.typed_get();
	
	let generated_spritesheet = server_state.timeline_generator.generate_image(media_path).await?;
	
	if let Some(if_modified_since) = if_modified_since {
		if !if_modified_since.is_modified(generated_spritesheet.mod_time) {
			let mut res = Response::builder()
				.status(StatusCode::NOT_MODIFIED)
				.body(Body::empty())
				.unwrap();
			
			res.headers_mut().typed_insert(LastModified::from(generated_spritesheet.mod_time));
			
			return Ok(res);
		}
	}
	
	let image_data = tokio::fs::read(&generated_spritesheet.cache_file).await?;
	
	let res = Response::builder()
		.header("Content-Type", mime::IMAGE_JPEG.essence_str())
		.body(Body::from(image_data))
		.unwrap();
	
	return Ok(res);
}