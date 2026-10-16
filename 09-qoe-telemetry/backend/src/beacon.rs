use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BeaconKind {
    Startup { startup_ms: u64 },
    RebufferStart,
    RebufferEnd { rebuffer_ms: u64 },
    BitrateSwitch { from_kbps: u32, to_kbps: u32 },
    Error { code: String },
    Heartbeat { played_ms: u64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Beacon {
    pub session_id: String,
    pub title: String,
    pub cdn: String,
    pub ts: i64,
    #[serde(flatten)]
    pub event: BeaconKind,
}

impl Beacon {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.session_id.is_empty() || self.title.is_empty() || self.cdn.is_empty() {
            return Err(AppError::InvalidBeacon(
                "session_id, title and cdn are required".into(),
            ));
        }
        if self.ts <= 0 {
            return Err(AppError::InvalidBeacon("ts must be epoch millis".into()));
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, AppError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json(raw: &[u8]) -> Result<Beacon, AppError> {
        Ok(serde_json::from_slice(raw)?)
    }
}
