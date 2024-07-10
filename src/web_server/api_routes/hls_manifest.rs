use http::{Method, Response};
use http::header::CONTENT_TYPE;
use tracing::instrument;

use crate::web_server::{libraries, video_locator};
use crate::web_server::api_error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::services::hls_segment_service;
use crate::web_server::web_utils::{full_body, HyperRequest, HyperResponse, restrict_method};

#[instrument(skip(server_state))]
pub async fn hls_manifest_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.iter().collect(), request.headers())?;
	let media_path = video_locator::locate_video(&resolved_path).await?.file()?;
	
	let media_metadata = server_state.video_metadata_cache.fetch_media_metadata(&media_path, &server_state.thumbnail_sheet_generator).await?;
	
	let mut manifest = String::new();
	
	manifest.push_str("#EXTM3U\n");
	
	if let Some(video_metadata) = &media_metadata.video_metadata {
		let levels = hls_segment_service::QUALITY_LEVELS.iter()
			.filter(|lvl| lvl.supported(video_metadata));
		
		for level in levels {
			manifest.push_str(&format!("#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={}x{},FRAME-RATE={}\n",
				level.max_bandwidth(),
				level.output_width(&video_metadata.video_size), level.target_video_height,
				f64::from(video_metadata.frame_rate)));
			
			manifest.push_str(&format!("level/{}/manifest.m3u8\n", level.id));
		}
	}
	
	Ok(Response::builder()
		.header(CONTENT_TYPE, "application/x-mpegURL")
		.body(full_body(manifest))
		.unwrap())
}