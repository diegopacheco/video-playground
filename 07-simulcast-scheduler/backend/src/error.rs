use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppError {
    #[error("episode {0} not found")]
    NotFound(String),
    #[error("invalid schedule: {0}")]
    InvalidSchedule(String),
    #[error("unknown time zone {0}")]
    UnknownTimeZone(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let code = match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InvalidSchedule(_) => StatusCode::BAD_REQUEST,
            AppError::UnknownTimeZone(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
