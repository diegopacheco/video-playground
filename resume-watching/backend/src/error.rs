use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("redis: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("scylla: {0}")]
    Scylla(String),
    #[error("codec: {0}")]
    Codec(#[from] serde_json::Error),
    #[error("config: {0}")]
    Config(String),
}

pub fn scylla_err<E: std::fmt::Display>(e: E) -> StoreError {
    StoreError::Scylla(e.to_string())
}
