use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid beacon: {0}")]
    InvalidBeacon(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("kafka error: {0}")]
    Kafka(#[from] rdkafka::error::KafkaError),
    #[error("topic creation failed: {0}")]
    Topic(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::InvalidBeacon(_) | AppError::Json(_) => StatusCode::BAD_REQUEST,
            AppError::Kafka(_) | AppError::Topic(_) | AppError::Io(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        (status, self.to_string()).into_response()
    }
}
