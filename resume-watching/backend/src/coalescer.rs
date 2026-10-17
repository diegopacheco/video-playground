use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use crate::error::StoreError;
use crate::model::Playhead;
use crate::store::PlayheadStore;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Offer {
    Buffered,
    Replaced,
    Stale,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoalescerStats {
    pub heartbeats: u64,
    pub stale: u64,
    pub writes: u64,
    pub flushes: u64,
    pub pending: usize,
}

#[derive(Default)]
pub struct Coalescer {
    pending: Mutex<HashMap<(String, String), Playhead>>,
    heartbeats: AtomicU64,
    stale: AtomicU64,
    writes: AtomicU64,
    flushes: AtomicU64,
}

impl Coalescer {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<(String, String), Playhead>> {
        self.pending.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn offer(&self, playhead: Playhead) -> Offer {
        self.heartbeats.fetch_add(1, Ordering::Relaxed);
        let outcome = merge(&mut self.lock(), playhead);
        if outcome == Offer::Stale {
            self.stale.fetch_add(1, Ordering::Relaxed);
        }
        outcome
    }

    pub fn drain(&self) -> Vec<Playhead> {
        self.lock().drain().map(|(_, v)| v).collect()
    }

    pub fn pending_for(&self, profile_id: &str) -> Vec<Playhead> {
        self.lock().values().filter(|p| p.profile_id == profile_id).cloned().collect()
    }

    pub fn forget_profile(&self, profile_id: &str) {
        self.lock().retain(|(profile, _), _| profile != profile_id);
    }

    pub fn requeue(&self, playheads: Vec<Playhead>) {
        let mut pending = self.lock();
        for p in playheads {
            merge(&mut pending, p);
        }
    }

    pub async fn flush(&self, store: &dyn PlayheadStore) -> Result<usize, StoreError> {
        let batch = self.drain();
        if batch.is_empty() {
            return Ok(0);
        }
        self.flushes.fetch_add(1, Ordering::Relaxed);
        for (i, p) in batch.iter().enumerate() {
            if let Err(e) = store.upsert(p).await {
                self.writes.fetch_add(i as u64, Ordering::Relaxed);
                self.requeue(batch[i..].to_vec());
                return Err(e);
            }
        }
        self.writes.fetch_add(batch.len() as u64, Ordering::Relaxed);
        Ok(batch.len())
    }

    pub fn stats(&self) -> CoalescerStats {
        CoalescerStats {
            heartbeats: self.heartbeats.load(Ordering::Relaxed),
            stale: self.stale.load(Ordering::Relaxed),
            writes: self.writes.load(Ordering::Relaxed),
            flushes: self.flushes.load(Ordering::Relaxed),
            pending: self.lock().len(),
        }
    }
}

fn merge(pending: &mut HashMap<(String, String), Playhead>, playhead: Playhead) -> Offer {
    match pending.get(&playhead.key()) {
        Some(current) if !playhead.wins_over(current) => Offer::Stale,
        Some(_) => {
            pending.insert(playhead.key(), playhead);
            Offer::Replaced
        }
        None => {
            pending.insert(playhead.key(), playhead);
            Offer::Buffered
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::playhead;
    use crate::store::StoreFuture;
    use std::sync::atomic::AtomicUsize;

    #[derive(Default)]
    struct MemoryStore {
        rows: Mutex<HashMap<(String, String), Playhead>>,
        upserts: AtomicUsize,
        fail_after: Option<usize>,
    }

    impl PlayheadStore for MemoryStore {
        fn name(&self) -> &'static str {
            "memory"
        }
        fn upsert<'a>(&'a self, p: &'a Playhead) -> StoreFuture<'a, ()> {
            Box::pin(async move {
                let n = self.upserts.fetch_add(1, Ordering::SeqCst);
                if self.fail_after.is_some_and(|limit| n >= limit) {
                    return Err(StoreError::Config("store down".into()));
                }
                self.rows.lock().unwrap().insert(p.key(), p.clone());
                Ok(())
            })
        }
        fn list<'a>(&'a self, profile_id: &'a str) -> StoreFuture<'a, Vec<Playhead>> {
            Box::pin(async move {
                Ok(self.rows.lock().unwrap().values().filter(|p| p.profile_id == profile_id).cloned().collect())
            })
        }
        fn clear<'a>(&'a self, _profile_id: &'a str) -> StoreFuture<'a, ()> {
            Box::pin(async move { Ok(()) })
        }
    }

    #[tokio::test]
    async fn six_heartbeats_inside_one_flush_window_cost_one_write() {
        let c = Coalescer::new();
        let store = MemoryStore::default();
        for i in 0..6 {
            c.offer(playhead("p1", "t1", "tv", i * 10, 1_000 + i64::from(i) * 10_000));
        }
        assert_eq!(c.flush(&store).await.unwrap(), 1);
        assert_eq!(store.upserts.load(Ordering::SeqCst), 1);
        let saved = store.list("p1").await.unwrap();
        assert_eq!(saved[0].position_secs, 50, "the flushed write must carry the latest position");
    }

    #[test]
    fn different_titles_of_same_profile_are_not_merged() {
        let c = Coalescer::new();
        c.offer(playhead("p1", "t1", "tv", 10, 1));
        c.offer(playhead("p1", "t2", "tv", 20, 2));
        assert_eq!(c.drain().len(), 2);
    }

    #[test]
    fn late_heartbeat_from_device_with_older_clock_loses() {
        let c = Coalescer::new();
        assert_eq!(c.offer(playhead("p1", "t1", "phone", 600, 2_000)), Offer::Buffered);
        assert_eq!(c.offer(playhead("p1", "t1", "tv", 100, 1_000)), Offer::Stale);
        let kept = c.drain();
        assert_eq!(kept[0].device_id, "phone");
        assert_eq!(kept[0].position_secs, 600);
        assert_eq!(c.stats().stale, 1);
    }

    #[test]
    fn newer_heartbeat_wins_even_when_it_rewinds_position() {
        let c = Coalescer::new();
        c.offer(playhead("p1", "t1", "tv", 900, 1_000));
        assert_eq!(c.offer(playhead("p1", "t1", "phone", 30, 2_000)), Offer::Replaced);
        assert_eq!(c.drain()[0].position_secs, 30, "user rewound on the phone, that is the truth");
    }

    #[tokio::test]
    async fn failed_flush_requeues_unwritten_positions_so_nothing_is_lost() {
        let c = Coalescer::new();
        let store = MemoryStore { fail_after: Some(1), ..Default::default() };
        c.offer(playhead("p1", "t1", "tv", 10, 1));
        c.offer(playhead("p1", "t2", "tv", 20, 2));
        assert!(c.flush(&store).await.is_err());
        let stats = c.stats();
        assert_eq!(stats.writes, 1);
        assert_eq!(stats.pending, 1);
    }

    #[test]
    fn requeue_does_not_clobber_newer_heartbeat_that_arrived_during_flush() {
        let c = Coalescer::new();
        c.offer(playhead("p1", "t1", "tv", 50, 5_000));
        c.requeue(vec![playhead("p1", "t1", "tv", 10, 1_000)]);
        assert_eq!(c.drain()[0].position_secs, 50);
    }

    #[test]
    fn forget_profile_drops_only_that_profile() {
        let c = Coalescer::new();
        c.offer(playhead("p1", "t1", "tv", 10, 1));
        c.offer(playhead("p2", "t1", "tv", 10, 1));
        c.forget_profile("p1");
        assert!(c.pending_for("p1").is_empty());
        assert_eq!(c.pending_for("p2").len(), 1);
    }
}
