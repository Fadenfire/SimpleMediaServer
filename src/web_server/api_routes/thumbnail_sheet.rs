use http::Method;
use tracing::instrument;

use crate::web_server::api_routes::error::ApiError;
use crate::web_server::server_state::ServerState;
use crate::web_server::{libraries, video_locator};
use crate::web_server::web_utils::{HyperRequest, HyperResponse, restrict_method, serve_file_basic};

#[instrument(skip(server_state, request))]
pub async fn thumbnail_sheet_route(
	server_state: &ServerState,
	request: &HyperRequest,
	library_id: &str,
	library_path: &[&str]
) -> Result<HyperResponse, ApiError> {
	restrict_method(request, &[Method::GET, Method::HEAD])?;
	
	let resolved_path = libraries::resolve_path_with_auth(
		server_state, library_id, library_path.iter().collect(), request.headers())?;
	let media_path = video_locator::locate_video(&resolved_path).await?.file()?;
	
	let generated_sprite_sheet = server_state.thumbnail_sheet_generator.get_or_generate(media_path).await?;
	
	let res = serve_file_basic(
		generated_sprite_sheet.entry_data,
		generated_sprite_sheet.creation_date.into(),
		mime::IMAGE_JPEG,
		request.headers()
	).await?;
	
	Ok(res)
}