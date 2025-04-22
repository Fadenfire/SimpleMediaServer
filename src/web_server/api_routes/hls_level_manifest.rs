use http::{Method, Response};
use http::header::CONTENT_TYPE;
use tracing::instrument;

use crate::web_server::{libraries, video_locator};
use crate::web_server::api_error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::services::hls_segment_service;
use crate::web_server::services::hls_segment_service::SEGMENT_DURATION;
use crate::web_server::web_utils::{full_body, HyperRequest, HyperResponse, restrict_method};

#[instrument(skip(server_state, request))]
pub async fn hls_level_manifest_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
	quality_level: &str,
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	hls_segment_service::get_quality_level(quality_level)?;
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.iter().collect(), request.headers())?;
	let media_path = video_locator::locate_video(&resolved_path).await?.file()?;
	
	let media_metadata = server_state.video_metadata_cache.fetch_basic_metadata(&media_path).await?;
	let duration = media_metadata.duration.as_secs_f64();
	
	let mut manifest = String::new();
	
	manifest.push_str("#EXTM3U\n");
	manifest.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");
	manifest.push_str(&format!("#EXT-X-TARGETDURATION:{}\n", SEGMENT_DURATION));
	manifest.push_str("#EXT-X-VERSION:4\n");
	manifest.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");
	
	let segments = (duration / SEGMENT_DURATION as f64).floor() as u32;
	
	for i in 0..segments {
		manifest.push_str(&format!("#EXTINF:{:?},\n", SEGMENT_DURATION as f64));
		manifest.push_str(&format!("segment/{}\n", i));
	}
	
	manifest.push_str(&format!("#EXTINF:{:?},\n", duration % SEGMENT_DURATION as f64));
	manifest.push_str(&format!("segment/{}\n", segments));
	
	manifest.push_str("#EXT-X-ENDLIST\n");
	
	Ok(Response::builder()
		.header(CONTENT_TYPE, "application/x-mpegURL")
		.body(full_body(manifest))
		.unwrap())
}