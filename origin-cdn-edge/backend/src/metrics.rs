use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct Metrics {
    pub requests: AtomicU64,
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub stale: AtomicU64,
    pub coalesced: AtomicU64,
    pub origin_fetches: AtomicU64,
    pub origin_errors: AtomicU64,
    pub revalidations: AtomicU64,
    pub evictions: AtomicU64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MetricsSnapshot {
    pub requests: u64,
    pub hits: u64,
    pub misses: u64,
    pub stale: u64,
    pub coalesced: u64,
    pub origin_fetches: u64,
    pub origin_errors: u64,
    pub revalidations: u64,
    pub evictions: u64,
    pub hit_ratio: f64,
    pub origin_offload: f64,
    pub cache_entries: usize,
    pub cache_bytes: usize,
    pub cache_capacity_bytes: usize,
    pub in_flight: usize,
}

pub struct CacheGauges {
    pub entries: usize,
    pub bytes: usize,
    pub capacity_bytes: usize,
    pub in_flight: usize,
}

fn ratio(part: u64, total: u64) -> f64 {
    if total == 0 { 0.0 } else { part as f64 / total as f64 }
}

impl Metrics {
    pub fn inc(counter: &AtomicU64) {
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add(counter: &AtomicU64, n: u64) {
        counter.fetch_add(n, Ordering::Relaxed);
    }

    fn counters(&self) -> [&AtomicU64; 9] {
        [
            &self.requests,
            &self.hits,
            &self.misses,
            &self.stale,
            &self.coalesced,
            &self.origin_fetches,
            &self.origin_errors,
            &self.revalidations,
            &self.evictions,
        ]
    }

    pub fn reset(&self) {
        for counter in self.counters() {
            counter.store(0, Ordering::Relaxed);
        }
    }

    pub fn snapshot(&self, gauges: CacheGauges) -> MetricsSnapshot {
        let [requests, hits, misses, stale, coalesced, origin_fetches, origin_errors, revalidations, evictions] =
            self.counters().map(|c| c.load(Ordering::Relaxed));
        MetricsSnapshot {
            requests,
            hits,
            misses,
            stale,
            coalesced,
            origin_fetches,
            origin_errors,
            revalidations,
            evictions,
            hit_ratio: ratio(hits + stale, requests),
            origin_offload: 1.0 - ratio(origin_fetches.min(requests), requests),
            cache_entries: gauges.entries,
            cache_bytes: gauges.bytes,
            cache_capacity_bytes: gauges.capacity_bytes,
            in_flight: gauges.in_flight,
        }
    }
}
