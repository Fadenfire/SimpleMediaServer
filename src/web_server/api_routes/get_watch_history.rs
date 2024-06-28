use http::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::api_routes::list_dir;
use crate::web_server::api_routes::list_dir::FileEntry;
use crate::web_server::server_state::ServerState;
use crate::web_server::{video_locator, web_utils};
use crate::web_server::video_locator::LocatedFile;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, json_response, restrict_method};

#[instrument(skip_all)]
pub async fn get_watch_history_route(server_state: &ServerState, request: &HyperRequest) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let params: Params = web_utils::parse_query(request.uri())?;
	let user = server_state.auth_manager.lookup_from_headers(request.headers())?;
	
	let mut entries = Vec::new();
	
	let history_entries: Vec<_> = {
		let mut user_watch_histories = server_state.user_watch_histories.lock().unwrap();
		
		user_watch_histories
			.get_watch_history(&user.id)
			.iter_entries()
			.rev()
			.skip(params.page * params.page_size)
			.filter(|entry| user.can_see_library(&entry.library_id))
			.filter_map(|entry| {
				server_state.libraries.resolve_path(&entry.library_id, entry.media_path.clone())
					.ok()
					.map(|path| (entry.clone(), path))
			})
			.take(params.page_size)
			.collect()
	};
	
	for (history_entry, resolved_path) in history_entries {
		let Ok(media_path) = video_locator::locate_video(&resolved_path).await.and_then(LocatedFile::file) else { continue };
		
		match list_dir::create_file_entry(server_state, user, &history_entry.library_id, &history_entry.media_path, &media_path).await {
			Ok(file_entry) => entries.push(file_entry),
			Err(err) => error!("Error collecting file metadata for {:?}: {:?}", &media_path, err),
		}
	}
	
	let res = WatchHistoryResponse {
		entries,
	};
	
	Ok(json_response(StatusCode::OK, &res))
}

#[derive(Debug, Deserialize)]
struct Params {
	page: usize,
	page_size: usize,
}

#[derive(Debug, Serialize)]
struct WatchHistoryResponse {
	entries: Vec<FileEntry>,
}
