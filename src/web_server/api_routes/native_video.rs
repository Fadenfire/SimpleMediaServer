use http::Method;
use http_body_util::BodyExt;
use tower_http::services::ServeFile;
use tracing::instrument;

use crate::web_server::api_error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::{libraries, video_locator};
use crate::web_server::web_utils::{HyperRequest, HyperResponse, restrict_method};

#[instrument(skip(server_state, request))]
pub async fn native_video_route(
	server_state: &ServerState,
	request: HyperRequest,
	library_id: &str,
	library_path: &[&str],
) -> Result<HyperResponse, ApiError> {
	restrict_method(&request, &[Method::GET, Method::HEAD])?;
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.iter().collect(), request.headers())?;
	let media_path = video_locator::locate_video(&resolved_path).await?.file()?;
	
	ServeFile::new(&media_path).try_call(request).await
		.map(|res| res.map(|body| body.map_err(anyhow::Error::new).boxed_unsync()))
		.map_err(ApiError::from)
}