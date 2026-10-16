#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("malformed mp4: {0}")]
    Mp4(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found")]
    NotFound,
    #[error("timed out waiting for media")]
    Timeout,
    #[error("source: {0}")]
    Source(String),
}

pub type Result<T> = std::result::Result<T, Error>;
