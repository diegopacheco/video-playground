use crate::clock::Clock;
use crate::error::RoomError;
use crate::playback::PlaybackState;
use crate::protocol::ServerMsg;
use crate::room::{MemberId, RoomRegistry, RoomView};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::mpsc::UnboundedSender;

pub type Outbox = UnboundedSender<ServerMsg>;

pub struct Hub {
    rooms: Mutex<Box<dyn RoomRegistry>>,
    peers: Mutex<HashMap<MemberId, Outbox>>,
    clock: Arc<dyn Clock>,
}

fn locked<T: ?Sized>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Hub {
    pub fn new(rooms: Box<dyn RoomRegistry>, clock: Arc<dyn Clock>) -> Self {
        Self {
            rooms: Mutex::new(rooms),
            peers: Mutex::new(HashMap::new()),
            clock,
        }
    }

    pub fn now_ms(&self) -> f64 {
        self.clock.now_ms()
    }

    pub fn join(&self, room: &str, name: &str, outbox: Outbox) -> Result<MemberId, RoomError> {
        let (id, view) = locked(&self.rooms).join(room, name, self.now_ms())?;
        let _ = outbox.send(ServerMsg::Welcome { you: id });
        locked(&self.peers).insert(id, outbox);
        self.broadcast(&view);
        Ok(id)
    }

    pub fn leave(&self, room: &str, member: MemberId) -> Result<(), RoomError> {
        locked(&self.peers).remove(&member);
        if let Some(view) = locked(&self.rooms).leave(room, member, self.now_ms())? {
            self.broadcast(&view);
        }
        Ok(())
    }

    pub fn set_state(&self, room: &str, member: MemberId, state: PlaybackState) -> Result<(), RoomError> {
        let view = locked(&self.rooms).set_state(room, member, state, self.now_ms())?;
        self.broadcast(&view);
        Ok(())
    }

    fn broadcast(&self, view: &RoomView) {
        let peers = locked(&self.peers);
        for member in &view.members {
            if let Some(outbox) = peers.get(&member.id) {
                let _ = outbox.send(ServerMsg::Room { view: view.clone() });
            }
        }
    }
}
