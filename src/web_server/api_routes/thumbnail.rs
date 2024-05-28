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
	headers: HeaderMap,
) -> Result<Response, ApiError> {
	let resolved_path = server_state.libraries.resolve_path(&library_path)?;
	let media_path = video_locator::locate_video(&resolved_path).await.map_err(|_| ApiError::FileNotFound)?;
	
	for ext in IMAGE_EXTENSIONS {
		let thumbnail_path = media_path.with_extension(ext);
		
		if let Some(thumbnail_metadata) = tokio::fs::metadata(&thumbnail_path).await.ok() {
			let mod_time = thumbnail_metadata.modified()?;
			
			let mime_type = mime_guess::from_path(&thumbnail_path).first_or_octet_stream();
			let res = serve_file_basic(&thumbnail_path, mod_time, mime_type, &headers).await?;
			
			return Ok(res);
		}
	}
	
	let generated_thumbnail = server_state.thumbnail_generator.extract_thumbnail(media_path).await?;
	let res = serve_file_basic(&generated_thumbnail.cache_file, generated_thumbnail.mod_time, mime::IMAGE_JPEG, &headers).await?;
	
	Ok(res)
}