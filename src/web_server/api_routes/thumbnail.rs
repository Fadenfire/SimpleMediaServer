use http::Method;
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::{libraries, video_locator};
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
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.iter().collect(), request.headers())?;
	let media_path = video_locator::locate_video(&resolved_path).await.map_err(|_| ApiError::FileNotFound)?;
	
	for ext in IMAGE_EXTENSIONS {
		let thumbnail_path = media_path.with_extension(ext);
		
		if let Some(thumbnail_metadata) = tokio::fs::metadata(&thumbnail_path).await.ok() {
			let mod_time = thumbnail_metadata.modified()?;
			
			let mime_type = mime_guess::from_path(&thumbnail_path).first_or_octet_stream();
			let data = tokio::fs::read(&thumbnail_path).await?;
			
			let res = serve_file_basic(data, mod_time, mime_type, request.headers()).await?;
			
			return Ok(res);
		}
	}
	
	let generated_thumbnail = server_state.thumbnail_generator.get_or_generate(media_path).await?;
	
	let res = serve_file_basic(
		generated_thumbnail.entry_data,
		generated_thumbnail.creation_date,
		mime::IMAGE_JPEG,
		request.headers()
	).await?;
	
	Ok(res)
}