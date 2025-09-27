use http::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

use crate::web_server::{video_locator, web_utils};
use crate::web_server::api_types::ApiWatchHistoryEntry;
use crate::web_server::api_error::ApiError;
use crate::web_server::api_routes::list_dir;
use crate::web_server::server_state::ServerState;
use crate::web_server::video_locator::LocatedFile;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, json_response, restrict_method};

#[instrument(skip_all)]
pub async fn get_watch_history_route(server_state: &ServerState, request: &HyperRequest) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let params: WatchHistoryParams = web_utils::parse_query(request.uri())?;
	let user = server_state.auth_manager.lookup_from_headers(request.headers())?;
	
	let history_entries: Vec<_>;
	let total_pages;
	
	{
		let mut user_watch_histories = server_state.user_watch_histories.lock().unwrap();
		let watch_history = user_watch_histories.get_watch_history(&user.id);
		
		if let Some(search_query) = &params.search_query {
			let search_query = search_query.to_lowercase();
			
			let mut iter = watch_history
				.iter_entries()
				.rev()
				.filter(|entry| {
					entry.media_path.file_name()
						.is_some_and(|file_name| file_name.to_lowercase().contains(&search_query))
				});
			
			let mut total_entries = (&mut iter).take(params.page * params.page_size).count();
			
			history_entries = (&mut iter)
				.take(params.page_size)
				.map(Clone::clone)
				.collect();
			
			total_entries += history_entries.len();
			total_entries += iter.count();
			
			total_pages = total_entries.div_ceil(params.page_size);
		} else {
			total_pages = watch_history.entry_count().div_ceil(params.page_size);
			
			history_entries = watch_history
				.iter_entries()
				.rev()
				.skip(params.page * params.page_size)
				.take(params.page_size)
				.map(Clone::clone)
				.collect();
		}
	};
	
	let mut entries = Vec::new();
	
	for entry in history_entries {
		let mut file_entry = None;
		
		if user.can_see_library(&entry.library_id) {
			if let Ok(resolved_path) = server_state.libraries.resolve_path(&entry.library_id, entry.media_path.clone()) {
				if let Ok(media_path) = video_locator::locate_video(&resolved_path).await.and_then(LocatedFile::file) {
					match list_dir::create_file_entry(server_state, user, &entry.library_id, &entry.media_path, &media_path).await {
						Ok(file) => file_entry = Some(file),
						Err(err) => error!("Error collecting file metadata for {:?}: {:?}", &media_path, err),
					}
				}
			}
		}
		
		entries.push(ApiWatchHistoryEntry {
			library_id: entry.library_id,
			media_path: entry.media_path,
			last_watched: entry.last_watched,
			progress: entry.progress,
			file: file_entry,
		})
	}
	
	let res = WatchHistoryResponse {
		total_pages,
		entries,
	};
	
	Ok(json_response(StatusCode::OK, &res))
}

#[derive(Debug, Deserialize)]
struct WatchHistoryParams {
	page: usize,
	page_size: usize,
	search_query: Option<String>,
}

#[derive(Debug, Serialize)]
struct WatchHistoryResponse {
	total_pages: usize,
	entries: Vec<ApiWatchHistoryEntry>,
}
