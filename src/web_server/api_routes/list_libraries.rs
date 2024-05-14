use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Serialize;
use tracing::instrument;

use crate::web_server::state::ServerState;

#[instrument(skip(server_state))]
pub async fn list_libraries_route(State(server_state): State<Arc<ServerState>>) -> Json<Vec<Library>> {
	let libraries: Vec<Library> = server_state.libraries.iter_libraries()
		.map(|lib| Library {
			id: lib.id.clone(),
			display_name: lib.display_name.clone(),
		})
		.collect();
	
	Json::from(libraries)
}

#[derive(Debug, Serialize)]
pub struct Library {
	id: String,
	display_name: String,
}