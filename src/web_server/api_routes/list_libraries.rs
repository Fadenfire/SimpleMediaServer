use std::sync::Arc;

use serde::Serialize;
use warp::{reply, Reply};

use crate::web_server::state::ServerState;

pub fn list_libraries_route(server_state: Arc<ServerState>) -> impl Reply {
	let libraries: Vec<LibraryResponse> = server_state.libraries.iter_libraries()
		.map(|lib| LibraryResponse {
			id: &lib.id,
			display_name: &lib.display_name,
		})
		.collect();
	
	reply::json(&libraries)
}

#[derive(Debug, Serialize)]
struct LibraryResponse<'a> {
	id: &'a str,
	display_name: &'a str,
}