use http::{Method, StatusCode};
use serde::Serialize;
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::libraries::Libraries;
use crate::web_server::web_utils::{HyperRequest, HyperResponse, json_response, restrict_method};

#[instrument(skip(request, libraries))]
pub async fn list_libraries_route(request: &HyperRequest, libraries: &Libraries) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let libraries: Vec<Library> = libraries.iter_libraries()
		.map(|lib| Library {
			id: lib.id.clone(),
			display_name: lib.display_name.clone(),
		})
		.collect();
	
	Ok(json_response(StatusCode::OK, &libraries))
}

#[derive(Debug, Serialize)]
pub struct Library {
	id: String,
	display_name: String,
}