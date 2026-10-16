use crate::error::RoomError;
use serde::{Deserialize, Serialize};

pub const MIN_RATE: f64 = 0.25;
pub const MAX_RATE: f64 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackState {
    pub playing: bool,
    pub position: f64,
    pub rate: f64,
    pub at: f64,
}

impl PlaybackState {
    pub fn paused_at_start(now_ms: f64) -> Self {
        Self {
            playing: false,
            position: 0.0,
            rate: 1.0,
            at: now_ms,
        }
    }

    pub fn expected_position(&self, now_ms: f64) -> f64 {
        if !self.playing {
            return self.position;
        }
        let elapsed_s = (now_ms - self.at).max(0.0) / 1000.0;
        self.position + elapsed_s * self.rate
    }

    pub fn rebased(&self, now_ms: f64) -> Self {
        Self {
            position: self.expected_position(now_ms),
            at: now_ms,
            ..*self
        }
    }

    pub fn validated(self, now_ms: f64) -> Result<Self, RoomError> {
        if !self.position.is_finite() || self.position < 0.0 {
            return Err(RoomError::InvalidState("position must be a finite number >= 0"));
        }
        if !self.rate.is_finite() || !(MIN_RATE..=MAX_RATE).contains(&self.rate) {
            return Err(RoomError::InvalidState("rate must be between 0.25 and 4"));
        }
        if !self.at.is_finite() {
            return Err(RoomError::InvalidState("at must be a finite server time"));
        }
        Ok(Self {
            at: self.at.min(now_ms),
            ..self
        })
    }
}
