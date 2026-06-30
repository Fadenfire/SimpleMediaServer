use std::str::FromStr;
use std::sync::Arc;
use http::Method;
use tracing::instrument;

use crate::web_server::services::hls_segment_service::SegmentParams;
use crate::web_server::api_error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::{libraries, video_locator};
use crate::web_server::media_metadata::AdvancedMediaMetadata;
use crate::web_server::services::hls_segment_service;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, restrict_method, serve_file_basic};

#[instrument(skip(server_state, request))]
pub async fn hls_segment_route(
	server_state: &Arc<ServerState>,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
	quality_level: &str,
	segment_index: &str,
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let segment_index: usize = segment_index.strip_suffix(".ts")
		.unwrap_or(segment_index)
		.parse()
		.map_err(|_| ApiError::NotFound)?;
	
	let quality_level = hls_segment_service::get_quality_level(quality_level)?;
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.iter().collect(), request.headers())?;
	let media_path = video_locator::locate_video(&resolved_path).await?.file()?;
	
	let advanced_metadata = server_state.metadata_cache
		.fetch_metadata::<AdvancedMediaMetadata>(&media_path).await?;
	
	if !hls_segment_service::is_segment_index_valid(segment_index, &advanced_metadata) {
		return Err(ApiError::InvalidSegmentIndex);
	}
	
	let params = SegmentParams {
		media_path,
		segment_index,
		quality_level,
	};
	
	let pending_query = server_state.hls_segment_generator
		.get_or_reserve(params.clone()).await?;
	
	let next_segment_index = segment_index + 1;
	
	// If there is a next segment, then start transcoding it in parallel to this segment
	if hls_segment_service::is_segment_index_valid(next_segment_index, &advanced_metadata) {
		let server_state = server_state.clone();
		
		tokio::task::spawn(async move {
			let _ = server_state.hls_segment_generator.get_or_reserve(SegmentParams {
				segment_index: next_segment_index,
				..params
			}).await;
		});
	}
	
	let generated_segment = pending_query.unwrap_or_generate().await?;
	
	let res = serve_file_basic(
		generated_segment.entry_data,
		generated_segment.creation_date.into(),
		mime::Mime::from_str("video/MP2T").unwrap(),
		request.headers()
	).await?;
	
	Ok(res)
}