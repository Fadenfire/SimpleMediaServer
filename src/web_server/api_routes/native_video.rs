use std::sync::Arc;
use axum::body::Body;
use axum::extract;
use axum::extract::{Request, State};
use axum::response::Response;
use tower_http::services::ServeFile;
use tracing::instrument;
use crate::web_server::api_routes::error::ApiError;
use crate::web_server::state::ServerState;
use crate::web_server::video_locator;

#[instrument(skip(server_state))]
pub async fn native_video_route(
	State(server_state): State<Arc<ServerState>>,
	extract::Path(library_path): extract::Path<String>,
	request: Request,
) -> Result<Response, ApiError> {
	let resolved_path = server_state.libraries.resolve_path(&library_path)?;
	let media_path = video_locator::locate_video(&resolved_path).await.map_err(|_| ApiError::FileNotFound)?;
	
	ServeFile::new(&media_path).try_call(request).await
		.map(|res| res.map(Body::new))
		.map_err(ApiError::from_io_error)
}