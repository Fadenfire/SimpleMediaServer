use http::{Method, Response, StatusCode};
use http::header::{LOCATION, SET_COOKIE};
use serde::Deserialize;
use tracing::instrument;

use crate::web_server::api_error::ApiError;
use crate::web_server::auth::{AUTH_COOKIE_NAME, AuthManager};
use crate::web_server::web_utils;
use crate::web_server::web_utils::{full_body, HyperRequest, HyperResponse, restrict_method};

#[instrument(skip_all)]
pub async fn login_route(request: HyperRequest, auth_manager: &AuthManager) -> Result<HyperResponse, ApiError> {
	restrict_method(&request, &[Method::POST])?;
	
	let params: LoginParams = web_utils::parse_form_body(request.into_body()).await?;
	
	let Ok(user) = auth_manager.login(&params.username, &params.password) else {
		let res = Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body(full_body("Invalid username/password"))
			.unwrap();
		
		return Ok(res)
	};
	
	let auth_token = auth_manager.generate_token(user);
	
	const ONE_YEAR: u32 = 60 * 60 * 24 * 365;
	
	let cookie = format!(
		"{name}={value}; Max-Age={age}; HttpOnly; SameSite=Strict",
		name = AUTH_COOKIE_NAME, value = auth_token, age = ONE_YEAR
	);
	
	let res = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(SET_COOKIE, cookie)
		.header(LOCATION, "/")
		.body(full_body("Redirecting"))
		.unwrap();
	
	Ok(res)
}

#[derive(Debug, Deserialize)]
struct LoginParams {
	username: String,
	password: String,
}
