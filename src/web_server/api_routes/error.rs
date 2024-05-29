use headers::HeaderMapExt;
use http::{Method, StatusCode};
use serde::Serialize;
use tracing::error;

use crate::web_server::web_utils::{HyperResponse, json_response};

pub enum ApiError {
	NotFound,
	LibraryNotFound,
	FileNotFound,
	NotADirectory,
	MethodNotAllowed(Vec<Method>),
	UnexpectedError(anyhow::Error),
}

impl ApiError {
	pub fn get_code_and_message(&self) -> (StatusCode, &str) {
		match self {
			Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
			Self::LibraryNotFound => (StatusCode::NOT_FOUND, "library_not_found"),
			Self::FileNotFound => (StatusCode::NOT_FOUND, "file_not_found"),
			Self::NotADirectory => (StatusCode::BAD_REQUEST, "not_a_directory"),
			Self::MethodNotAllowed(_) => (StatusCode::METHOD_NOT_ALLOWED, "method_not_allowed"),
			Self::UnexpectedError(err) => {
				error!("Unexpected error: {:?}", err);
				(StatusCode::INTERNAL_SERVER_ERROR, "unexpected_error")
			}
		}
	}
	
	pub fn into_response(self) -> HyperResponse {
		let (status, message) = self.get_code_and_message();
		
		let reply = ErrorMessage {
			code: status.as_u16(),
			message: message.to_owned(),
		};
		
		let mut res = json_response(status, &reply);
		
		if let Self::MethodNotAllowed(allowed_methods) = self {
			res.headers_mut().typed_insert(headers::Allow::from_iter(allowed_methods));
		}
		
		res
	}
}

impl From<anyhow::Error> for ApiError {
	fn from(err: anyhow::Error) -> Self {
		if let Some(err) = err.downcast_ref::<std::io::Error>() {
			use std::io::ErrorKind;
			
			if let ErrorKind::NotFound | ErrorKind::PermissionDenied = err.kind() {
				return Self::FileNotFound;
			}
		}
		
		Self::UnexpectedError(err)
	}
}

impl From<std::io::Error> for ApiError {
	fn from(err: std::io::Error) -> Self {
		use std::io::ErrorKind;
		
		match err.kind() {
			ErrorKind::NotFound | ErrorKind::PermissionDenied => Self::FileNotFound,
			_ => Self::UnexpectedError(err.into()),
		}
	}
}

#[derive(Serialize)]
struct ErrorMessage {
	code: u16,
	message: String,
}
