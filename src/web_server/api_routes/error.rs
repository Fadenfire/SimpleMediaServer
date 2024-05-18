use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::error;

pub enum ApiError {
	NotFound,
	LibraryNotFound,
	FileNotFound,
	NotADirectory,
	UnexpectedError(anyhow::Error),
}

impl ApiError {
	pub fn get_code_and_message(&self) -> (StatusCode, &str) {
		match self {
			Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
			Self::LibraryNotFound => (StatusCode::NOT_FOUND, "library_not_found"),
			Self::FileNotFound => (StatusCode::NOT_FOUND, "file_not_found"),
			Self::NotADirectory => (StatusCode::BAD_REQUEST, "not_a_directory"),
			Self::UnexpectedError(err) => {
				error!("Unexpected error: {:?}", err);
				(StatusCode::INTERNAL_SERVER_ERROR, "unexpected_error")
			},
		}
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
