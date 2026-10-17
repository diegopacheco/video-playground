use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum DrmError {
    #[error("malformed: {0}")]
    Malformed(String),
    #[error("bad signature")]
    BadSignature,
    #[error("token expired")]
    Expired,
    #[error("missing credentials")]
    Unauthenticated,
    #[error("not entitled to {0}")]
    NotEntitled(String),
    #[error("unknown key id {0}")]
    UnknownKey(String),
    #[error("not found")]
    NotFound,
    #[error("config: {0}")]
    Config(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl DrmError {
    pub fn code(&self) -> &'static str {
        match self {
            DrmError::Malformed(_) => "malformed",
            DrmError::BadSignature => "bad_signature",
            DrmError::Expired => "expired",
            DrmError::Unauthenticated => "unauthenticated",
            DrmError::NotEntitled(_) => "not_entitled",
            DrmError::UnknownKey(_) => "unknown_key",
            DrmError::NotFound => "not_found",
            DrmError::Config(_) => "config",
            DrmError::Io(_) => "io",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            DrmError::Malformed(_) => StatusCode::BAD_REQUEST,
            DrmError::BadSignature | DrmError::Expired | DrmError::NotEntitled(_) => {
                StatusCode::FORBIDDEN
            }
            DrmError::Unauthenticated => StatusCode::UNAUTHORIZED,
            DrmError::UnknownKey(_) | DrmError::NotFound => StatusCode::NOT_FOUND,
            DrmError::Config(_) | DrmError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for DrmError {
    fn into_response(self) -> Response {
        let body = Json(json!({ "error": self.code(), "message": self.to_string() }));
        (self.status(), body).into_response()
    }
}
