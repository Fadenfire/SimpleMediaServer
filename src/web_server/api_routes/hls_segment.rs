use std::str::FromStr;
use http::Method;
use tracing::instrument;

use crate::web_server::services::hls_segment_service::SegmentParams;
use crate::web_server::api_error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::{libraries, video_locator};
use crate::web_server::services::hls_segment_service;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, restrict_method, serve_file_basic};

#[instrument(skip(server_state, request))]
pub async fn hls_segment_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
	quality_level: &str,
	segment_index: &str,
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let segment_index: usize = segment_index.parse().map_err(|_| ApiError::NotFound)?;
	let quality_level = hls_segment_service::get_quality_level(quality_level)?;
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.iter().collect(), request.headers())?;
	let media_path = video_locator::locate_video(&resolved_path).await?.file()?;
	
	let generated_segment = server_state.hls_segment_generator.get_or_generate(SegmentParams {
		media_path,
		segment_index,
		quality_level,
	}).await?;
	
	let res = serve_file_basic(
		generated_segment.entry_data,
		generated_segment.creation_date.into(),
		mime::Mime::from_str("video/MP2T").unwrap(),
		request.headers()
	).await?;
	
	Ok(res)
}