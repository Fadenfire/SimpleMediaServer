use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::{anyhow, Context};
use bytes::Bytes;
use flate2::write::GzEncoder;
use headers::{HeaderMap, HeaderMapExt, IfModifiedSince, LastModified};
use http::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE};
use http::{Method, Request, Response, StatusCode, Uri};
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::Incoming;
use mime::Mime;
use relative_path::RelativePath;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::web_server::api_error::ApiError;

pub type HyperRequest = Request<Incoming>;
pub type HyperResponse = Response<HyperBody>;
pub type HyperBody = UnsyncBoxBody<Bytes, anyhow::Error>;

pub fn empty_body() -> HyperBody {
	Empty::<Bytes>::new()
		.map_err(|never| match never {})
		.boxed_unsync()
}

pub fn full_body<T: Into<Bytes>>(chunk: T) -> HyperBody {
	Full::new(chunk.into())
		.map_err(|never| match never {})
		.boxed_unsync()
}


pub fn simple_json_response<T: Serialize>(status_code: StatusCode, json: &T) -> HyperResponse {
	let data = serde_json::to_vec(json).expect("Error serializing object");
	
	Response::builder()
		.status(status_code)
		.header(CONTENT_TYPE, mime::APPLICATION_JSON.essence_str())
		.body(full_body(data))
		.unwrap()
}

pub async fn json_response<T: Serialize>(
	json: &T,
	request_headers: &HeaderMap,
) -> anyhow::Result<HyperResponse> {
	let data = serde_json::to_vec(json).expect("Error serializing object");
	
	let builder = Response::builder()
		.status(StatusCode::OK)
		.header(CONTENT_TYPE, mime::APPLICATION_JSON.essence_str());
	
	finish_compressed_response(builder, data, request_headers).await
}

pub fn restrict_method(request: &HyperRequest, allowed_methods: &[Method]) -> Result<(), ApiError> {
	if allowed_methods.contains(request.method()) {
		Ok(())
	} else {
		Err(ApiError::MethodNotAllowed(allowed_methods.to_vec()))
	}
}

pub async fn collect_body(body: Incoming) -> Result<Bytes, ApiError> {
	http_body_util::Limited::new(body, 1_000_000)
		.collect()
		.await
		.map(|collected| collected.to_bytes())
		.map_err(|_| ApiError::InvalidBody)
}

pub async fn parse_json_body<T: DeserializeOwned>(body: Incoming) -> Result<T, ApiError> {
	collect_body(body).await
		.and_then(|data| serde_json::from_slice(&data).map_err(|_| ApiError::InvalidBody))
}

pub async fn parse_form_body<T: DeserializeOwned>(body: Incoming) -> Result<T, ApiError> {
	collect_body(body).await
		.and_then(|data| serde_urlencoded::from_bytes(&data).map_err(|_| ApiError::InvalidBody))
}

pub fn parse_query<T: DeserializeOwned>(uri: &Uri) -> Result<T, ApiError> {
	uri.query()
		.and_then(|query| serde_urlencoded::from_str(query).ok())
		.ok_or(ApiError::InvalidQuery)
}

const COMPRESSION_LEVEL: flate2::Compression = flate2::Compression::new(3);
const COMPRESSION_THRESHOLD: usize = 8 * 1024; // 8 KiB

pub async fn finish_compressed_response(
	builder: http::response::Builder,
	body_data: impl Into<Bytes>,
	request_headers: &HeaderMap
) -> anyhow::Result<HyperResponse> {
	let accept_encoding = request_headers.get(ACCEPT_ENCODING);
	// This is a hack, but `headers` doesn't have a struct for AcceptEncoding
	let accepts_gzip = accept_encoding
		.and_then(|header| header.to_str().ok())
		.is_some_and(|value| value.contains("gzip"));
	
	let body_data = body_data.into();
	
	if accepts_gzip && body_data.len() > COMPRESSION_THRESHOLD {
		let gzipped_data = tokio::task::spawn_blocking(move || {
			let mut encoder = GzEncoder::new(Vec::new(), COMPRESSION_LEVEL);
			encoder.write_all(&body_data).unwrap();
			
			encoder.finish().unwrap()
		}).await.context("Compression failed")?;
		
		let res = builder
			.header(CONTENT_ENCODING, "gzip")
			.body(full_body(gzipped_data))
			.unwrap();
		
		Ok(res)
	} else {
		let res = builder
			.body(full_body(body_data))
			.unwrap();
		
		Ok(res)
	}
}

fn check_if_modified_since(
	mod_time: SystemTime,
	request_headers: &HeaderMap,
) -> anyhow::Result<Option<HyperResponse>> {
	let if_modified_since: Option<IfModifiedSince> = request_headers.typed_get();
	
	if let Some(if_modified_since) = if_modified_since {
		if !if_modified_since.is_modified(mod_time) {
			let mut res = Response::builder()
				.status(StatusCode::NOT_MODIFIED)
				.body(empty_body())
				.unwrap();
			
			res.headers_mut().typed_insert(LastModified::from(mod_time));
			
			return Ok(Some(res));
		}
	}
	
	Ok(None)
}

pub async fn serve_file_basic(
	file_data: impl Into<Bytes>,
	mod_time: SystemTime,
	mime_type: Mime,
	request_headers: &HeaderMap
) -> anyhow::Result<HyperResponse> {
	if let Some(res) = check_if_modified_since(mod_time, request_headers)? {
		return Ok(res);
	}
	
	let res = Response::builder()
		.header(CONTENT_TYPE, mime_type.essence_str())
		.body(full_body(file_data))
		.unwrap();
	
	Ok(res)
}

pub async fn serve_file_compressed(
	file_data: impl Into<Bytes>,
	mod_time: SystemTime,
	mime_type: Mime,
	request_headers: &HeaderMap
) -> anyhow::Result<HyperResponse> {
	if let Some(res) = check_if_modified_since(mod_time, request_headers)? {
		return Ok(res);
	}
	
	let builder = Response::builder()
		.header(CONTENT_TYPE, mime_type.essence_str());
	
	finish_compressed_response(builder, file_data, request_headers).await
}

pub fn split_path(path: &str) -> anyhow::Result<Vec<String>> {
	if !path.starts_with('/') {
		return Err(anyhow!("Path doesn't begin with a slash"));
	}
	
	let mut components: Vec<String> = Vec::new();
	
	// [1..] slices off the leading slash.
	for component in path[1..].split('/') {
		let component = percent_encoding::percent_decode_str(component)
			.decode_utf8()
			.context("Decoding path segment")?;
		
		match component.as_ref() {
			"." | "" => {}
			".." => { components.pop(); }
			_ => components.push(component.into())
		}
	}
	
	// if path.ends_with('/') {
	// 	components.push("".to_string());
	// }
	
	Ok(components)
}

pub fn sanitize_path(path: &RelativePath) -> Option<PathBuf> {
	let mut out_path = PathBuf::new();
	
	for component in path.components() {
		let relative_path::Component::Normal(component) = component else { return None };
		
		if component.starts_with("..") || component.contains(['\\', '/']) {
			return None;
		}
		
		if component.is_empty() || component == "." {
			continue;
		}
		
		out_path.push(component);
	}
	
	if out_path.is_absolute() {
		return None;
	}
	
	if !out_path.components().all(|c| matches!(c, std::path::Component::Normal(_))) {
		return None;
	}
	
	Some(out_path)
}
