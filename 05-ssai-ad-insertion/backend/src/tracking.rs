use std::collections::HashSet;
use std::sync::Mutex;

use serde::Serialize;

use crate::ads::EPSILON;
use crate::error::{Result, SsaiError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AdEvent {
    Impression,
    FirstQuartile,
    Midpoint,
    ThirdQuartile,
    Complete,
}

const MILESTONES: [(AdEvent, f64); 5] = [
    (AdEvent::Impression, 0.0),
    (AdEvent::FirstQuartile, 0.25),
    (AdEvent::Midpoint, 0.5),
    (AdEvent::ThirdQuartile, 0.75),
    (AdEvent::Complete, 1.0),
];

pub fn events_for_segment(durations: &[f64], index: usize) -> Vec<AdEvent> {
    if index >= durations.len() {
        return Vec::new();
    }
    let total: f64 = durations.iter().sum();
    let start: f64 = durations[..index].iter().sum();
    let end = start + durations[index];
    MILESTONES
        .iter()
        .filter(|(_, fraction)| {
            let point = fraction * total;
            let reached_here = point <= end + EPSILON;
            let reached_before = index > 0 && point <= start + EPSILON;
            reached_here && !reached_before
        })
        .map(|(event, _)| *event)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Beacon {
    pub seq: u64,
    pub user: String,
    pub session: String,
    pub break_id: String,
    pub creative: String,
    pub event: AdEvent,
    pub at_ms: u128,
}

pub trait BeaconSink: Send + Sync {
    fn record(&self, beacon: Beacon) -> Result<bool>;
    fn list(&self, session: Option<&str>) -> Result<Vec<Beacon>>;
    fn clear(&self) -> Result<()>;
}

#[derive(Default)]
pub struct MemoryBeaconSink {
    inner: Mutex<(Vec<Beacon>, HashSet<String>)>,
}

impl BeaconSink for MemoryBeaconSink {
    fn record(&self, mut beacon: Beacon) -> Result<bool> {
        let mut guard = self.inner.lock().map_err(|_| SsaiError::Lock)?;
        let key = format!("{}|{}|{}|{:?}", beacon.session, beacon.break_id, beacon.creative, beacon.event);
        if !guard.1.insert(key) {
            return Ok(false);
        }
        beacon.seq = guard.0.len() as u64 + 1;
        guard.0.push(beacon);
        Ok(true)
    }

    fn list(&self, session: Option<&str>) -> Result<Vec<Beacon>> {
        let guard = self.inner.lock().map_err(|_| SsaiError::Lock)?;
        Ok(guard.0.iter().filter(|b| session.is_none_or(|s| b.session == s)).cloned().collect())
    }

    fn clear(&self) -> Result<()> {
        let mut guard = self.inner.lock().map_err(|_| SsaiError::Lock)?;
        guard.0.clear();
        guard.1.clear();
        Ok(())
    }
}
