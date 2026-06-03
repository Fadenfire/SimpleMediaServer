use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;
use bytes::Bytes;
use http::Method;
use mime::Mime;
use relative_path::RelativePath;
use tracing::instrument;

use crate::web_server::api_error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::web_utils::{restrict_method, serve_file_basic, HyperRequest, HyperResponse};
use crate::web_server::{libraries, video_locator};
use crate::web_server::services::artifact_cache::ArtifactCache;
use crate::web_server::services::thumbnail_service::ThumbnailGenerator;
use crate::web_server::video_locator::LocatedFile;

const IMAGE_EXTENSIONS: &[&str] = &[
	"jpg",
	"jpeg",
	"png",
	"webp"
];

pub fn create_full_thumbnail_path(library_path: &RelativePath) -> String {
	format!("/api/thumbnail/{}", library_path)
}

pub fn create_scaled_thumbnail_path(library_path: &RelativePath) -> String {
	format!("/api/scaled_thumbnail/{}", library_path)
}

#[instrument(skip(server_state, request))]
pub async fn full_thumbnail_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.iter().collect(), request.headers())?;
	
	let located_file = video_locator::locate_video(&resolved_path).await?;
	
	let thumbnail = get_thumbnail(located_file, &server_state.thumbnail_generator).await?
		.ok_or_else(|| ApiError::FileNotFound)?;
	
	let res = serve_file_basic(
		thumbnail.file_data,
		thumbnail.mod_time,
		thumbnail.mime_type,
		request.headers()
	).await?;
	
	Ok(res)
}

#[instrument(skip(server_state, request))]
pub async fn scaled_thumbnail_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.iter().collect(), request.headers())?;
	
	let located_file = video_locator::locate_video(&resolved_path).await?;
	
	let full_thumbnail = get_thumbnail(located_file, &server_state.thumbnail_generator).await?
		.ok_or_else(|| ApiError::FileNotFound)?;
	
	let scaled_thumbnail = server_state.scaled_thumbnail_generator
		.get_or_generate(full_thumbnail.file_data).await?;
	
	let res = serve_file_basic(
		scaled_thumbnail.entry_data,
		scaled_thumbnail.creation_date.into(),
		Mime::from_str("image/webp").unwrap(),
		request.headers()
	).await?;
	
	Ok(res)
}

struct ThumbnailFile {
	pub file_data: Bytes,
	pub mod_time: SystemTime,
	pub mime_type: Mime,
}

async fn get_thumbnail(
	located_file: LocatedFile,
	thumbnail_generator: &ArtifactCache<ThumbnailGenerator>,
) -> anyhow::Result<Option<ThumbnailFile>> {
	match located_file {
		LocatedFile::File(media_path) => {
			if let Some(file) = find_first_image(&media_path).await? {
				Ok(Some(file))
			} else {
				let generated_thumbnail = thumbnail_generator.get_or_generate(media_path).await?;
				
				Ok(Some(ThumbnailFile {
					file_data: generated_thumbnail.entry_data,
					mod_time: generated_thumbnail.creation_date.into(),
					mime_type: mime::IMAGE_JPEG,
				}))
			}
		}
		LocatedFile::Directory(dir_path) => {
			find_first_image(&dir_path.join("thumbnail")).await
		}
	}
}

async fn find_first_image(path: &Path) -> anyhow::Result<Option<ThumbnailFile>> {
	for ext in IMAGE_EXTENSIONS {
		let thumbnail_path = path.with_extension(ext);
		
		if let Some(thumbnail_metadata) = tokio::fs::metadata(&thumbnail_path).await.ok() {
			let mod_time = thumbnail_metadata.modified()?;
			
			let mime_type = mime_guess::from_path(&thumbnail_path).first_or_octet_stream();
			let data = tokio::fs::read(&thumbnail_path).await?;
			
			return Ok(Some(ThumbnailFile {
				file_data: data.into(),
				mod_time,
				mime_type,
			}));
		}
	}
	
	Ok(None)
}