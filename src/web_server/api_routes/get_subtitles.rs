use std::str::FromStr;
use http::Method;
use tracing::instrument;

use crate::web_server::api_error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::{libraries, video_locator};
use crate::web_server::services::subtitle_service::SubtitleParams;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, restrict_method, serve_file_basic};

#[instrument(skip(server_state, request))]
pub async fn subtitles_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str],
	stream_index: &str,
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let stream_index: usize = stream_index.parse().map_err(|_| ApiError::NotFound)?;
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.iter().collect(), request.headers())?;
	let media_path = video_locator::locate_video(&resolved_path).await?.file()?;
	
	let subtitles = server_state.transcoded_subtitle_generator.get_or_generate(SubtitleParams {
		media_path,
		stream_index,
	}).await?;
	
	let res = serve_file_basic(
		subtitles.entry_data,
		subtitles.creation_date.into(),
		mime::Mime::from_str("text/vtt").unwrap(),
		request.headers()
	).await?;
	
	Ok(res)
}