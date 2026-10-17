use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("ffmpeg failed: {0}")]
    Ffmpeg(String),
    #[error("could not parse {metric} score from ffmpeg output")]
    Parse { metric: String },
    #[error("invalid config: {0}")]
    Config(String),
    #[error("analysis already running")]
    Busy,
}

pub type Result<T> = std::result::Result<T, AppError>;
