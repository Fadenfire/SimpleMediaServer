use std::path::Path;

use http::{HeaderMap, Method};
use relative_path::RelativePath;
use tracing::instrument;

use crate::web_server::{libraries, video_locator};
use crate::web_server::api_error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::video_locator::LocatedFile;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, restrict_method, serve_file_basic};

const IMAGE_EXTENSIONS: &[&str] = &[
	"jpg",
	"jpeg",
	"png",
	"webp"
];

pub fn create_thumbnail_path(library_path: &RelativePath) -> String {
	format!("/api/thumbnail/{}", library_path)
}

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
	
	match video_locator::locate_video(&resolved_path).await? {
		LocatedFile::File(media_path) => {
			if let Some(res) = serve_first_image(&media_path, request.headers()).await? {
				Ok(res)
			} else {
				let generated_thumbnail = server_state.thumbnail_generator.get_or_generate(media_path).await?;
				
				let res = serve_file_basic(
					generated_thumbnail.entry_data,
					generated_thumbnail.creation_date.into(),
					mime::IMAGE_JPEG,
					request.headers()
				).await?;
				
				Ok(res)
			}
		}
		LocatedFile::Directory(dir_path) => {
			serve_first_image(&dir_path.join("thumbnail"), request.headers()).await?
				.ok_or_else(|| ApiError::FileNotFound)
		}
	}
}

async fn serve_first_image(path: &Path, headers: &HeaderMap) -> Result<Option<HyperResponse>, ApiError> {
	for ext in IMAGE_EXTENSIONS {
		let thumbnail_path = path.with_extension(ext);
		
		if let Some(thumbnail_metadata) = tokio::fs::metadata(&thumbnail_path).await.ok() {
			let mod_time = thumbnail_metadata.modified()?;
			
			let mime_type = mime_guess::from_path(&thumbnail_path).first_or_octet_stream();
			let data = tokio::fs::read(&thumbnail_path).await?;
			
			let res = serve_file_basic(data, mod_time, mime_type, headers).await?;
			
			return Ok(Some(res));
		}
	}
	
	Ok(None)
}