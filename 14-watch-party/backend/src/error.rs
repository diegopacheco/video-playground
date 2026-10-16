use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum RoomError {
    #[error("room {0} does not exist")]
    UnknownRoom(String),
    #[error("member {0} is not in the room")]
    UnknownMember(u64),
    #[error("only the leader can change playback")]
    NotLeader,
    #[error("invalid playback state: {0}")]
    InvalidState(&'static str),
    #[error("invalid join: {0}")]
    InvalidJoin(&'static str),
}
