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
	
	let mut thumbnail_path = None;
	
	for ext in IMAGE_EXTENSIONS {
		let path = media_path.with_extension(ext);
		
		if tokio::fs::try_exists(&path).await.unwrap_or(false) {
			thumbnail_path = Some(path);
			break;
		}
	}
	
	let if_modified_since: Option<IfModifiedSince> = headers.typed_get();
	
	if let Some(thumbnail_path) = thumbnail_path {
		let thumbnail_metadata = tokio::fs::metadata(&thumbnail_path).await?;
		let mod_time = thumbnail_metadata.modified()?;
		
		if let Some(if_modified_since) = if_modified_since {
			if !if_modified_since.is_modified(mod_time) {
				let res = Response::builder()
					.status(StatusCode::NOT_MODIFIED)
					.body(Body::empty())
					.unwrap();
				
				return Ok(res);
			}
		}
		
		let mime_type = mime_guess::from_path(&thumbnail_path).first_or_octet_stream();
		let image_data = tokio::fs::read(&thumbnail_path).await.map_err(|_| ApiError::FileNotFound)?;
		
		let mut res = Response::builder()
			.header("Content-Type", mime_type.essence_str())
			.body(Body::from(image_data))
			.unwrap();
		
		res.headers_mut().typed_insert(LastModified::from(mod_time));
		
		return Ok(res);
	}
	
	let generated_thumbnail = server_state.thumbnail_extractor.extract_thumbnail(media_path).await?;
	
	if let Some(if_modified_since) = if_modified_since {
		if !if_modified_since.is_modified(generated_thumbnail.mod_time) {
			let mut res = Response::builder()
				.status(StatusCode::NOT_MODIFIED)
				.body(Body::empty())
				.unwrap();
			
			res.headers_mut().typed_insert(LastModified::from(generated_thumbnail.mod_time));
			
			return Ok(res);
		}
	}
	
	let image_data = tokio::fs::read(&generated_thumbnail.cache_file).await?;
	
	let res = Response::builder()
		.header("Content-Type", mime::IMAGE_JPEG.essence_str())
		.body(Body::from(image_data))
		.unwrap();
	
	return Ok(res);
}