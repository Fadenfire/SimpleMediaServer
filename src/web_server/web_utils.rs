use std::path::Path;
use std::time::SystemTime;

use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::Response;
use headers::{HeaderMap, HeaderMapExt, IfModifiedSince, LastModified};
use mime::Mime;

pub async fn serve_file_basic(file_path: &Path, mod_time: SystemTime, mime_type: Mime, request_headers: &HeaderMap) -> anyhow::Result<Response> {
	let if_modified_since: Option<IfModifiedSince> = request_headers.typed_get();
	
	if let Some(if_modified_since) = if_modified_since {
		if !if_modified_since.is_modified(mod_time) {
			let mut res = Response::builder()
				.status(StatusCode::NOT_MODIFIED)
				.body(Body::empty())
				.unwrap();
			
			res.headers_mut().typed_insert(LastModified::from(mod_time));
			
			return Ok(res);
		}
	}
	
	let data = tokio::fs::read(file_path).await?;
	
	let res = Response::builder()
		.header(CONTENT_TYPE, mime_type.essence_str())
		.body(Body::from(data))
		.unwrap();
	
	Ok(res)
}