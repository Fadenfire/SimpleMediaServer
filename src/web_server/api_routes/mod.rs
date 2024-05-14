mod list_libraries;

use std::convert::Infallible;
use std::sync::Arc;
use serde::Serialize;

use warp::{Filter, Rejection, Reply, reply};
use warp::filters::BoxedFilter;
use warp::http::StatusCode;

use crate::web_server::state::ServerState;

pub fn build_routes(server_state: Arc<ServerState>) -> BoxedFilter<(impl Reply, )> {
	let server_state_clone = server_state.clone();
	let list_libraries = warp::get()
		.map(move || server_state_clone.clone())
		.and(warp::path!("libraries"))
		.map(list_libraries::list_libraries_route);
	
	list_libraries
		.recover(handle_rejection)
		.boxed()
}

async fn handle_rejection(rejection: Rejection) -> Result<impl Reply, Rejection> {
	let code;
	let message;
	
	if rejection.is_not_found() {
		code = StatusCode::NOT_FOUND;
		message = "NOT_FOUND".to_owned();
	} else {
		return Err(rejection);
	}
	
	let json = reply::json(&ErrorMessage {
		code: code.as_u16(),
		message,
	});
	
	Ok(reply::with_status(json, code))
}

#[derive(Serialize)]
struct ErrorMessage {
	code: u16,
	message: String,
}
