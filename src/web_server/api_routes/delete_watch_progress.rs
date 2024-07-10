use http::{Method, Response};
use relative_path::RelativePathBuf;
use serde::Deserialize;
use tracing::instrument;

use crate::web_server::api_error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::web_utils;
use crate::web_server::web_utils::{empty_body, HyperRequest, HyperResponse, restrict_method};

#[instrument(skip_all)]
pub async fn delete_watch_progress_route(server_state: &ServerState, request: HyperRequest) -> Result<HyperResponse, ApiError> {
	restrict_method(&request, &[Method::POST])?;
	
	let (request, body) = request.into_parts();
	let params: DeleteWatchHistoryParams = web_utils::parse_json_body(body).await?;
	
	let user = server_state.auth_manager.lookup_from_headers(&request.headers)?;
	
	{
		let mut user_watch_histories = server_state.user_watch_histories.lock().unwrap();
		
		user_watch_histories.get_watch_history(&user.id)
			.delete_entry(&params.library_id, &params.media_path);
		
		user_watch_histories.mark_dirty();
	}
	
	Ok(Response::new(empty_body()))
}

#[derive(Debug, Deserialize)]
struct DeleteWatchHistoryParams {
	pub library_id: String,
	pub media_path: RelativePathBuf,
}
