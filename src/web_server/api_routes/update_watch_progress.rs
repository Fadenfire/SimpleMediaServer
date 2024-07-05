use http::{Method, Response};
use relative_path::RelativePathBuf;
use serde::Deserialize;
use tracing::instrument;

use crate::web_server::{libraries, video_locator, web_utils};
use crate::web_server::api_routes::error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::web_utils::{empty_body, HyperRequest, HyperResponse, restrict_method};

#[instrument(skip_all)]
pub async fn update_watch_progress_route(server_state: &ServerState, request: HyperRequest) -> Result<HyperResponse, ApiError> {
	restrict_method(&request, &[Method::POST])?;
	
	let (request, body) = request.into_parts();
	let params: UpdateWatchHistoryParams = web_utils::parse_json_body(body).await?;
	
	let user = server_state.auth_manager.lookup_from_headers(&request.headers)?;
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, &params.library_id, params.media_path.clone(), &request.headers)?;
	
	if video_locator::locate_video(&resolved_path).await.is_err() {
		return Err(ApiError::FileNotFound);
	}
	
	{
		let mut user_watch_histories = server_state.user_watch_histories.lock().unwrap();
		
		user_watch_histories.get_watch_history(&user.id)
			.update_progress(&params.library_id, &params.media_path, params.new_watch_progress);
		
		user_watch_histories.mark_dirty();
	}
	
	Ok(Response::new(empty_body()))
}

#[derive(Debug, Deserialize)]
struct UpdateWatchHistoryParams {
	pub library_id: String,
	pub media_path: RelativePathBuf,
	pub new_watch_progress: u64,
}
