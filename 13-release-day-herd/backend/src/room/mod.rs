use std::collections::HashMap;

use serde::Serialize;

use crate::HerdError;
use crate::admission::{AdmissionGate, max_in_window};
use crate::rng::Rng;

const MICROS_PER_SECOND: u64 = 1_000_000;
const MANIFEST: &str = "/cdn/release/master.m3u8";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JoinResponse {
    pub ticket: u64,
    pub position: u64,
    pub eta_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StatusResponse {
    pub ticket: u64,
    pub position: u64,
    pub eta_ms: u64,
    pub admitted: bool,
    pub token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlayResponse {
    pub ticket: u64,
    pub manifest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoomStats {
    pub rate_per_s: u64,
    pub joined: u64,
    pub admitted: u64,
    pub waiting: u64,
    pub played: u64,
    pub max_admitted_in_any_second: u64,
    pub max_plays_in_any_second: u64,
}

pub struct LiveRoom {
    gate: AdmissionGate,
    rng: Rng,
    secrets: Vec<u64>,
    owners: HashMap<u64, u64>,
    plays: Vec<u64>,
}

impl LiveRoom {
    pub fn new(rate_per_s: u64, seed: u64) -> Result<Self, HerdError> {
        Ok(Self {
            gate: AdmissionGate::new(rate_per_s)?,
            rng: Rng::new(seed),
            secrets: Vec::new(),
            owners: HashMap::new(),
            plays: Vec::new(),
        })
    }

    pub fn join(&mut self, now_us: u64) -> JoinResponse {
        let ticket = self.gate.join(now_us);
        let secret = self.rng.next_u64();
        self.secrets.push(secret);
        self.owners.insert(secret, ticket.index);
        JoinResponse { ticket: ticket.index, position: ticket.position, eta_ms: ticket.eta_us / 1_000 }
    }

    pub fn status(&self, ticket: u64, now_us: u64) -> Result<StatusResponse, HerdError> {
        let at = self.gate.admit_at(ticket).ok_or(HerdError::TicketNotFound(ticket))?;
        let position = self.gate.position(ticket, now_us).ok_or(HerdError::TicketNotFound(ticket))?;
        let admitted = at <= now_us;
        Ok(StatusResponse {
            ticket,
            position,
            eta_ms: at.saturating_sub(now_us) / 1_000,
            admitted,
            token: admitted.then(|| format!("{:016x}", self.secrets[ticket as usize])),
        })
    }

    pub fn play(&mut self, token: &str, now_us: u64) -> Result<PlayResponse, HerdError> {
        let secret = u64::from_str_radix(token, 16).map_err(|_| HerdError::NotAdmitted)?;
        let ticket = *self.owners.get(&secret).ok_or(HerdError::NotAdmitted)?;
        let at = self.gate.admit_at(ticket).ok_or(HerdError::TicketNotFound(ticket))?;
        if at > now_us {
            return Err(HerdError::NotAdmitted);
        }
        self.plays.push(now_us);
        Ok(PlayResponse { ticket, manifest: MANIFEST.to_string() })
    }

    pub fn stats(&self, now_us: u64) -> RoomStats {
        let admitted = self.gate.admitted_times(now_us);
        RoomStats {
            rate_per_s: self.gate.rate_per_s(),
            joined: self.gate.joined(),
            admitted: admitted.len() as u64,
            waiting: self.gate.joined() - admitted.len() as u64,
            played: self.plays.len() as u64,
            max_admitted_in_any_second: max_in_window(admitted, MICROS_PER_SECOND),
            max_plays_in_any_second: max_in_window(&self.plays, MICROS_PER_SECOND),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nobody_plays_before_the_room_lets_them_in() {
        let mut room = LiveRoom::new(2, 9).unwrap();
        room.join(0);
        let second = room.join(0);
        assert_eq!(second.eta_ms, 500);
        assert!(room.status(second.ticket, 100_000).unwrap().token.is_none());
        let forged = format!("{:016x}", room.secrets[1]);
        assert_eq!(room.play(&forged, 100_000), Err(HerdError::NotAdmitted));
        let token = room.status(second.ticket, 500_000).unwrap().token.unwrap();
        assert!(room.play(&token, 500_000).is_ok());
    }

    #[test]
    fn garbage_tokens_and_unknown_tickets_are_refused() {
        let mut room = LiveRoom::new(2, 9).unwrap();
        assert_eq!(room.play("zz", 0), Err(HerdError::NotAdmitted));
        assert_eq!(room.status(4, 0), Err(HerdError::TicketNotFound(4)));
    }

    #[test]
    fn stats_prove_the_live_room_respects_its_rate() {
        let mut room = LiveRoom::new(50, 1).unwrap();
        for _ in 0..400 {
            room.join(0);
        }
        let stats = room.stats(10 * MICROS_PER_SECOND);
        assert_eq!(stats.admitted, 400);
        assert!(stats.max_admitted_in_any_second <= 50);
    }
}
