use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum AbrError {
    #[error("trace {trace} line {line}: {reason}")]
    TraceParse {
        trace: String,
        line: usize,
        reason: String,
    },
    #[error("trace {0} has no positive bandwidth sample")]
    EmptyTrace(String),
    #[error("unknown trace {0}")]
    UnknownTrace(String),
    #[error("unknown algorithm {0}")]
    UnknownAlgorithm(String),
    #[error("no algorithm requested")]
    NoAlgorithms,
    #[error("invalid player config: {0}")]
    InvalidConfig(String),
    #[error("algorithm {algorithm} chose quality {quality} outside the ladder")]
    InvalidQuality { algorithm: String, quality: usize },
}
