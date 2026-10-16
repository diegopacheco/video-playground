use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum SubError {
    #[error("line {line}: {msg}")]
    Parse { line: usize, msg: String },
    #[error("invalid timestamp '{0}'")]
    Timestamp(String),
    #[error("invalid color '{0}'")]
    Color(String),
    #[error("unknown locale '{0}'")]
    UnknownLocale(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("media not ready: {0}")]
    MediaMissing(String),
}

impl SubError {
    pub fn parse(line: usize, msg: impl Into<String>) -> Self {
        SubError::Parse {
            line,
            msg: msg.into(),
        }
    }
}
