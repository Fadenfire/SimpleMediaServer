use http::Method;
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::router::ServerState;
use crate::web_server::video_locator;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, restrict_method, serve_file_basic};

const IMAGE_EXTENSIONS: &[&str] = &[
	"jpg",
	"jpeg",
	"png",
	"webp"
];

#[instrument(skip(server_state, request))]
pub async fn thumbnail_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let resolved_path = server_state.libraries.resolve_path(library_id, library_path)?;
	let media_path = video_locator::locate_video(&resolved_path).await.map_err(|_| ApiError::FileNotFound)?;
	
	for ext in IMAGE_EXTENSIONS {
		let thumbnail_path = media_path.with_extension(ext);
		
		if let Some(thumbnail_metadata) = tokio::fs::metadata(&thumbnail_path).await.ok() {
			let mod_time = thumbnail_metadata.modified()?;
			
			let mime_type = mime_guess::from_path(&thumbnail_path).first_or_octet_stream();
			let res = serve_file_basic(&thumbnail_path, mod_time, mime_type, request.headers()).await?;
			
			return Ok(res);
		}
	}
	
	let generated_thumbnail = server_state.thumbnail_generator.get_or_generate(media_path).await?;
	let res = serve_file_basic(&generated_thumbnail.cache_file, generated_thumbnail.mod_time, mime::IMAGE_JPEG, request.headers()).await?;
	
	Ok(res)
}