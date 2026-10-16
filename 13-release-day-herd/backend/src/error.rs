use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HerdError {
    #[error("unknown strategy: {0}")]
    UnknownStrategy(String),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("ticket not found: {0}")]
    TicketNotFound(u64),
    #[error("token not admitted")]
    NotAdmitted,
    #[error("simulation worker failed: {0}")]
    Worker(String),
}
