use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub enum ApiError {
	NotFound,
}

impl ApiError {
	pub fn get_code_and_message(&self) -> (StatusCode, &str) {
		match self {
			Self::NotFound => (StatusCode::NOT_FOUND, "not_found")
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
