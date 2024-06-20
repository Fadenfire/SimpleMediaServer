use std::str::FromStr;
use http::Method;
use tracing::instrument;

use crate::web_server::services::hls_segment_service::SegmentParams;
use crate::web_server::api_routes::error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::video_locator;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, restrict_method, serve_file_basic};

#[instrument(skip(server_state, request))]
pub async fn hls_segment_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
	segment_index: &str,
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let segment_index: usize = segment_index.parse().map_err(|_| ApiError::NotFound)?;
	let resolved_path = server_state.libraries.resolve_path(library_id, library_path)?;
	let media_path = video_locator::locate_video(&resolved_path).await.map_err(|_| ApiError::FileNotFound)?;
	
	let generated_segment = server_state.hls_segment_generator.get_or_generate(SegmentParams {
		media_path,
		segment_index,
	}).await?;
	
	let res = serve_file_basic(
		generated_segment.entry_data,
		generated_segment.creation_date,
		mime::Mime::from_str("video/MP2T").unwrap(),
		request.headers()
	).await?;
	
	Ok(res)
}