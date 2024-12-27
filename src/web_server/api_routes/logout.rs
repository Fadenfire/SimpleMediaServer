use http::header::{LOCATION, SET_COOKIE};
use http::{Method, Response, StatusCode};
use tracing::instrument;

use crate::web_server::api_error::ApiError;
use crate::web_server::auth::AUTH_COOKIE_NAME;
use crate::web_server::web_utils::{full_body, restrict_method, HyperRequest, HyperResponse};

#[instrument(skip_all)]
pub async fn logout_route(request: &HyperRequest) -> Result<HyperResponse, ApiError> {
	restrict_method(&request, &[Method::GET])?;
	
	let cookie = format!(
		"{name}=; Max-Age=-100; HttpOnly; SameSite=Strict",
		name = AUTH_COOKIE_NAME
	);
	
	let res = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(SET_COOKIE, cookie)
		.header(LOCATION, "/")
		.body(full_body("Redirecting"))
		.unwrap();
	
	Ok(res)
}
