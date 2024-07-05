use http::{Method, StatusCode};
use tracing::instrument;

use crate::web_server::api_routes::api_types::ApiLibraryEntry;
use crate::web_server::api_routes::error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, json_response, restrict_method};

#[instrument(skip_all)]
pub async fn list_libraries_route(server_state: &ServerState, request: &HyperRequest) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let user = server_state.auth_manager.lookup_from_headers(request.headers())?;
	
	let libraries: Vec<_> = server_state.libraries.iter_libraries()
		.filter(|lib| user.can_see_library(&lib.id))
		.map(|lib| ApiLibraryEntry {
			id: lib.id.clone(),
			display_name: lib.display_name.clone(),
		})
		.collect();
	
	Ok(json_response(StatusCode::OK, &libraries))
}
