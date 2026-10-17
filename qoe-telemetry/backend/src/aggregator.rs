use std::collections::{BTreeMap, HashSet};

use serde::Serialize;

use crate::beacon::{Beacon, BeaconKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    Cdn,
    Title,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WindowMetrics {
    pub sessions: usize,
    pub events: u64,
    pub startups: usize,
    pub avg_startup_ms: f64,
    pub p95_startup_ms: f64,
    pub rebuffer_ratio: f64,
    pub switches_per_min: f64,
    pub error_rate: f64,
    pub played_ms: u64,
    pub rebuffer_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WindowResult {
    pub window_start: i64,
    pub window_end: i64,
    pub dimension: Dimension,
    pub key: String,
    pub metrics: WindowMetrics,
}

pub trait Aggregator: Send {
    fn ingest(&mut self, beacon: &Beacon) -> bool;
    fn advance(&mut self, watermark_ms: i64) -> Vec<WindowResult>;
    fn late_dropped(&self) -> u64;
}

#[derive(Default)]
struct Accumulator {
    sessions: HashSet<String>,
    error_sessions: HashSet<String>,
    startups: Vec<u64>,
    played_ms: u64,
    rebuffer_ms: u64,
    switches: u64,
    events: u64,
}

impl Accumulator {
    fn add(&mut self, beacon: &Beacon) {
        self.events += 1;
        self.sessions.insert(beacon.session_id.clone());
        match &beacon.event {
            BeaconKind::Startup { startup_ms } => self.startups.push(*startup_ms),
            BeaconKind::RebufferEnd { rebuffer_ms } => self.rebuffer_ms += rebuffer_ms,
            BeaconKind::BitrateSwitch { .. } => self.switches += 1,
            BeaconKind::Error { .. } => {
                self.error_sessions.insert(beacon.session_id.clone());
            }
            BeaconKind::Heartbeat { played_ms } => self.played_ms += played_ms,
            BeaconKind::RebufferStart => {}
        }
    }

    fn metrics(mut self) -> WindowMetrics {
        self.startups.sort_unstable();
        let watch_ms = self.played_ms + self.rebuffer_ms;
        WindowMetrics {
            sessions: self.sessions.len(),
            events: self.events,
            startups: self.startups.len(),
            avg_startup_ms: mean(&self.startups),
            p95_startup_ms: percentile(&self.startups, 0.95),
            rebuffer_ratio: ratio(self.rebuffer_ms as f64, watch_ms as f64),
            switches_per_min: ratio(self.switches as f64, self.played_ms as f64 / 60_000.0),
            error_rate: ratio(self.error_sessions.len() as f64, self.sessions.len() as f64),
            played_ms: self.played_ms,
            rebuffer_ms: self.rebuffer_ms,
        }
    }
}

fn ratio(num: f64, den: f64) -> f64 {
    if den <= 0.0 { 0.0 } else { num / den }
}

pub fn mean(sorted: &[u64]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted.iter().sum::<u64>() as f64 / sorted.len() as f64
}

pub fn percentile(sorted: &[u64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (p * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1] as f64
}

type WindowKey = (i64, Dimension, String);

pub struct TumblingWindows {
    size_ms: i64,
    open: BTreeMap<WindowKey, Accumulator>,
    watermark: i64,
    late: u64,
}

impl TumblingWindows {
    pub fn new(size_ms: i64) -> Self {
        Self {
            size_ms,
            open: BTreeMap::new(),
            watermark: i64::MIN,
            late: 0,
        }
    }

    pub fn window_start(&self, ts: i64) -> i64 {
        ts - ts.rem_euclid(self.size_ms)
    }

    fn slot(&mut self, start: i64, dimension: Dimension, key: &str) -> &mut Accumulator {
        self.open.entry((start, dimension, key.to_string())).or_default()
    }
}

impl Aggregator for TumblingWindows {
    fn ingest(&mut self, beacon: &Beacon) -> bool {
        let start = self.window_start(beacon.ts);
        if start + self.size_ms <= self.watermark {
            self.late += 1;
            return false;
        }
        self.slot(start, Dimension::Cdn, &beacon.cdn).add(beacon);
        self.slot(start, Dimension::Title, &beacon.title).add(beacon);
        true
    }

    fn advance(&mut self, watermark_ms: i64) -> Vec<WindowResult> {
        self.watermark = self.watermark.max(watermark_ms);
        let size = self.size_ms;
        let ready: Vec<WindowKey> = self
            .open
            .keys()
            .take_while(|(start, _, _)| start + size <= self.watermark)
            .cloned()
            .collect();
        ready
            .into_iter()
            .filter_map(|key| {
                let acc = self.open.remove(&key)?;
                let (start, dimension, name) = key;
                Some(WindowResult {
                    window_start: start,
                    window_end: start + size,
                    dimension,
                    key: name,
                    metrics: acc.metrics(),
                })
            })
            .collect()
    }

    fn late_dropped(&self) -> u64 {
        self.late
    }
}
