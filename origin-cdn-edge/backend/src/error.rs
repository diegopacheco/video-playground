use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CdnError {
    #[error("origin unreachable: {0}")]
    OriginUnreachable(String),
    #[error("origin returned status {0}")]
    OriginStatus(u16),
    #[error("invalid configuration: {0}")]
    Config(String),
}
