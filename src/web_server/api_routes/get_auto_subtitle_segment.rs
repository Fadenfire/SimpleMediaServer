use http::Method;
use std::str::FromStr;
use tracing::instrument;

use crate::web_server::api_error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::services::transcription_service::AutoSubtitleParams;
use crate::web_server::web_utils::{restrict_method, serve_file_compressed, HyperRequest, HyperResponse};
use crate::web_server::{libraries, video_locator};

#[instrument(skip(server_state, request))]
pub async fn auto_subtitle_segment_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
	segment_index: &str,
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let segment_index: usize = segment_index.strip_suffix(".webvtt")
		.unwrap_or(segment_index)
		.parse()
		.map_err(|_| ApiError::NotFound)?;
	
	let Some(auto_subtitle_generator) = &server_state.auto_subtitle_generator else {
		return Err(ApiError::FeatureNotSupported);
	};
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.iter().collect(), request.headers())?;
	let media_path = video_locator::locate_video(&resolved_path).await?.file()?;
	
	let subtitles = auto_subtitle_generator.get_or_generate(AutoSubtitleParams {
		media_path,
		segment_index,
	}).await?;
	
	let res = serve_file_compressed(
		subtitles.entry_data,
		subtitles.creation_date.into(),
		mime::Mime::from_str("text/vtt").unwrap(),
		request.headers()
	).await?;
	
	Ok(res)
}