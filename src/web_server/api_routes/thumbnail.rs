use std::sync::Arc;

use axum::body::Body;
use axum::extract;
use axum::extract::State;
use axum::response::Response;
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::state::ServerState;
use crate::web_server::video_locator;

const IMAGE_EXTENSIONS: &[&str] = &[
	"jpg",
	"jpeg",
	"png",
	"webp"
];

#[instrument(skip(server_state))]
pub async fn thumbnail_route(
	State(server_state): State<Arc<ServerState>>,
	extract::Path(library_path): extract::Path<String>,
) -> Result<Response, ApiError> {
	let resolved_path = server_state.libraries.resolve_path(&library_path)?;
	let media_path = video_locator::locate_video(&resolved_path).await.map_err(|_| ApiError::FileNotFound)?;
	
	let mut thumbnail_path = None;
	
	for ext in IMAGE_EXTENSIONS {
		let path = media_path.with_extension(ext);
		
		if tokio::fs::try_exists(&path).await.unwrap_or(false) {
			thumbnail_path = Some(path);
			break;
		}
	}
	
	if let Some(thumbnail_path) = thumbnail_path {
		let mime_type = mime_guess::from_path(&thumbnail_path).first_or_octet_stream();
		let image_data = tokio::fs::read(&thumbnail_path).await.map_err(|_| ApiError::FileNotFound)?;
		
		let res = Response::builder()
			.header("Content-Type", mime_type.essence_str())
			.body(Body::from(image_data))
			.unwrap();
		
		return Ok(res);
	}
	
	Err(ApiError::FileNotFound)
}