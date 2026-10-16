use crate::error::RoomError;
use crate::playback::PlaybackState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type MemberId = u64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: MemberId,
    pub name: String,
    pub joined_at: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomView {
    pub room: String,
    pub leader: MemberId,
    pub members: Vec<Member>,
    pub state: PlaybackState,
}

pub trait RoomRegistry: Send {
    fn join(&mut self, room: &str, name: &str, now_ms: f64) -> Result<(MemberId, RoomView), RoomError>;
    fn leave(&mut self, room: &str, member: MemberId, now_ms: f64) -> Result<Option<RoomView>, RoomError>;
    fn set_state(
        &mut self,
        room: &str,
        member: MemberId,
        state: PlaybackState,
        now_ms: f64,
    ) -> Result<RoomView, RoomError>;
    fn view(&self, room: &str) -> Result<RoomView, RoomError>;
}

struct Room {
    members: Vec<Member>,
    leader: MemberId,
    state: PlaybackState,
}

impl Room {
    fn view(&self, name: &str) -> RoomView {
        RoomView {
            room: name.to_string(),
            leader: self.leader,
            members: self.members.clone(),
            state: self.state,
        }
    }
}

#[derive(Default)]
pub struct InMemoryRooms {
    rooms: HashMap<String, Room>,
    next_id: MemberId,
}

impl InMemoryRooms {
    pub fn new() -> Self {
        Self::default()
    }
}

const MAX_NAME_LEN: usize = 32;

impl RoomRegistry for InMemoryRooms {
    fn join(&mut self, room: &str, name: &str, now_ms: f64) -> Result<(MemberId, RoomView), RoomError> {
        let room = room.trim();
        let name = name.trim();
        if room.is_empty() || room.len() > MAX_NAME_LEN {
            return Err(RoomError::InvalidJoin("room must have 1 to 32 characters"));
        }
        if name.is_empty() || name.len() > MAX_NAME_LEN {
            return Err(RoomError::InvalidJoin("name must have 1 to 32 characters"));
        }
        self.next_id += 1;
        let id = self.next_id;
        let member = Member {
            id,
            name: name.to_string(),
            joined_at: now_ms,
        };
        let entry = self.rooms.entry(room.to_string()).or_insert_with(|| Room {
            members: Vec::new(),
            leader: id,
            state: PlaybackState::paused_at_start(now_ms),
        });
        entry.members.push(member);
        Ok((id, entry.view(room)))
    }

    fn leave(&mut self, room: &str, member: MemberId, now_ms: f64) -> Result<Option<RoomView>, RoomError> {
        let entry = self
            .rooms
            .get_mut(room)
            .ok_or_else(|| RoomError::UnknownRoom(room.to_string()))?;
        let index = entry
            .members
            .iter()
            .position(|m| m.id == member)
            .ok_or(RoomError::UnknownMember(member))?;
        entry.members.remove(index);
        let Some(next) = entry.members.first().map(|m| m.id) else {
            self.rooms.remove(room);
            return Ok(None);
        };
        if entry.leader == member {
            entry.leader = next;
            entry.state = entry.state.rebased(now_ms);
        }
        Ok(Some(entry.view(room)))
    }

    fn set_state(
        &mut self,
        room: &str,
        member: MemberId,
        state: PlaybackState,
        now_ms: f64,
    ) -> Result<RoomView, RoomError> {
        let entry = self
            .rooms
            .get_mut(room)
            .ok_or_else(|| RoomError::UnknownRoom(room.to_string()))?;
        if !entry.members.iter().any(|m| m.id == member) {
            return Err(RoomError::UnknownMember(member));
        }
        if entry.leader != member {
            return Err(RoomError::NotLeader);
        }
        entry.state = state.validated(now_ms)?;
        Ok(entry.view(room))
    }

    fn view(&self, room: &str) -> Result<RoomView, RoomError> {
        self.rooms
            .get(room)
            .map(|r| r.view(room))
            .ok_or_else(|| RoomError::UnknownRoom(room.to_string()))
    }
}
