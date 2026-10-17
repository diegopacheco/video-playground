#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("command `{command}` failed: {stderr}")]
    Command { command: String, stderr: String },
    #[error("parse error: {0}")]
    Parse(String),
    #[error("invalid ladder: {0}")]
    Ladder(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid path: {0}")]
    InvalidPath(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
