use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::error;

pub enum ApiError {
	NotFound,
	LibraryNotFound,
	FileNotFound,
	UnexpectedError,
}

impl ApiError {
	pub fn get_code_and_message(&self) -> (StatusCode, &str) {
		match self {
			Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
			Self::LibraryNotFound => (StatusCode::NOT_FOUND, "library_not_found"),
			Self::FileNotFound => (StatusCode::NOT_FOUND, "file_not_found"),
			Self::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, "unexpected_error"),
		}
	}
	
	pub fn from_io_error(err: std::io::Error) -> Self {
		use std::io::ErrorKind;
		
		match err.kind() {
			ErrorKind::NotFound | ErrorKind::PermissionDenied => Self::FileNotFound,
			_ => {
				error!("IO error: {:?}", err);
				Self::UnexpectedError
			},
		}
	}
	
	pub fn from_unknown_err(err: anyhow::Error) -> Self {
		error!("Unknown error: {:?}", err);
		Self::UnexpectedError
	}
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		let (status, message) = self.get_code_and_message();
		
		let reply = ErrorMessage {
			code: status.as_u16(),
			message: message.to_owned(),
		};
		
		(status, Json(reply)).into_response()
	}
}

#[derive(Serialize)]
struct ErrorMessage {
	code: u16,
	message: String,
}
