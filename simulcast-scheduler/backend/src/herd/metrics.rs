use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct Metrics {
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub throttled: AtomicU64,
    pub plays_granted: AtomicU64,
    pub plays_throttled: AtomicU64,
    pub warmed: AtomicU64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub throttled: u64,
    pub plays_granted: u64,
    pub plays_throttled: u64,
    pub warmed: u64,
    pub cache_entries: usize,
    pub bucket_tokens: f64,
}

pub fn bump(counter: &AtomicU64) {
    counter.fetch_add(1, Ordering::Relaxed);
}

impl Metrics {
    pub fn snapshot(&self, cache_entries: usize, bucket_tokens: f64) -> MetricsSnapshot {
        let read = |c: &AtomicU64| c.load(Ordering::Relaxed);
        MetricsSnapshot {
            cache_hits: read(&self.cache_hits),
            cache_misses: read(&self.cache_misses),
            throttled: read(&self.throttled),
            plays_granted: read(&self.plays_granted),
            plays_throttled: read(&self.plays_throttled),
            warmed: read(&self.warmed),
            cache_entries,
            bucket_tokens,
        }
    }

    pub fn reset(&self) {
        for c in [
            &self.cache_hits,
            &self.cache_misses,
            &self.throttled,
            &self.plays_granted,
            &self.plays_throttled,
            &self.warmed,
        ] {
            c.store(0, Ordering::Relaxed);
        }
    }
}
