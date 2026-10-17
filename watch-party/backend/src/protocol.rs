use crate::room::{MemberId, RoomView};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ClientMsg {
    Join { room: String, name: String },
    Ping { t0: f64 },
    State { playing: bool, position: f64, rate: f64, at: f64 },
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServerMsg {
    Welcome { you: MemberId },
    Pong { t0: f64, t1: f64, t2: f64 },
    Room { view: RoomView },
    Error { message: String },
}
