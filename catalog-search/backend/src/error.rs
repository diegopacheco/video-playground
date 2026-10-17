use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("query is empty after normalization")]
    EmptyQuery,
    #[error("catalog is invalid: {0}")]
    InvalidCatalog(String),
    #[error("catalog json could not be parsed: {0}")]
    Parse(#[from] serde_json::Error),
}
