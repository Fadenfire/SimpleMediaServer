use http::Method;
use tracing::instrument;

use crate::web_server::api_error::ApiError;
use crate::web_server::api_types::ApiUserInfo;
use crate::web_server::auth::AuthManager;
use crate::web_server::web_utils::{large_json_response, restrict_method, HyperRequest, HyperResponse};

#[instrument(skip_all)]
pub async fn get_user_route(request: &HyperRequest, auth_manager: &AuthManager) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let user = auth_manager.lookup_from_headers(request.headers())?;
	
	let user_res = ApiUserInfo {
		display_name: user.display_name.clone(),
		username: user.username.clone(),
	};
	
	Ok(large_json_response(&user_res, request.headers()).await?)
}
