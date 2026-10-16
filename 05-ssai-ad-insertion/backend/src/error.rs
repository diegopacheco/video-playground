use thiserror::Error;

#[derive(Debug, Error)]
pub enum SsaiError {
    #[error("invalid playlist: {0}")]
    InvalidPlaylist(String),
    #[error("invalid catalog: {0}")]
    InvalidCatalog(String),
    #[error("unknown user: {0}")]
    UnknownUser(String),
    #[error("unknown creative: {0}")]
    UnknownCreative(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("lock poisoned")]
    Lock,
}

pub type Result<T> = std::result::Result<T, SsaiError>;
