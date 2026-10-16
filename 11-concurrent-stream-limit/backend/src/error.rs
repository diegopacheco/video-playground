use thiserror::Error;

#[derive(Debug, Error)]
pub enum LeaseError {
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("unknown account {0}")]
    UnknownAccount(String),
    #[error("invalid device id")]
    InvalidDevice,
    #[error("invalid policy {0}")]
    InvalidPolicy(String),
    #[error("unexpected store reply: {0}")]
    BadReply(String),
    #[error("store lock poisoned")]
    Poisoned,
}
