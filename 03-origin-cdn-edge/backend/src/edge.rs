use crate::cache::Cache;
use crate::cache_control::CachePolicy;
use crate::coalescer::{Coalescer, Role};
use crate::entry::{Entry, Freshness};
use crate::error::CdnError;
use crate::metrics::{CacheGauges, Metrics, MetricsSnapshot};
use crate::origin_client::OriginClient;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStatus {
    Hit,
    Miss,
    Stale,
}

impl CacheStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheStatus::Hit => "HIT",
            CacheStatus::Miss => "MISS",
            CacheStatus::Stale => "STALE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EdgeResponse {
    pub entry: Entry,
    pub status: CacheStatus,
    pub age: Duration,
    pub coalesced: bool,
}

pub struct Edge<C: Cache, O: OriginClient> {
    cache: Mutex<C>,
    origin: O,
    coalescer: Coalescer,
    revalidating: Mutex<HashSet<String>>,
    metrics: Metrics,
}

impl<C: Cache, O: OriginClient> Edge<C, O> {
    pub fn new(cache: C, origin: O) -> Arc<Self> {
        Arc::new(Edge {
            cache: Mutex::new(cache),
            origin,
            coalescer: Coalescer::default(),
            revalidating: Mutex::new(HashSet::new()),
            metrics: Metrics::default(),
        })
    }

    pub fn metrics(&self) -> MetricsSnapshot {
        let (entries, bytes, capacity_bytes) = self
            .cache
            .lock()
            .map(|c| (c.len(), c.bytes(), c.capacity_bytes()))
            .unwrap_or((0, 0, 0));
        self.metrics.snapshot(CacheGauges {
            entries,
            bytes,
            capacity_bytes,
            in_flight: self.coalescer.in_flight(),
        })
    }

    pub fn reset_metrics(&self) {
        self.metrics.reset();
    }

    fn lookup(&self, key: &str) -> Option<Entry> {
        self.cache.lock().ok().and_then(|mut c| c.get(key))
    }

    pub async fn get(self: &Arc<Self>, key: &str) -> Result<EdgeResponse, CdnError> {
        Metrics::inc(&self.metrics.requests);
        let now = Instant::now();
        if let Some(entry) = self.lookup(key) {
            match entry.freshness(now) {
                Freshness::Fresh => {
                    Metrics::inc(&self.metrics.hits);
                    return Ok(EdgeResponse { age: entry.age(now), entry, status: CacheStatus::Hit, coalesced: false });
                }
                Freshness::Stale => {
                    Metrics::inc(&self.metrics.stale);
                    self.revalidate_in_background(key);
                    return Ok(EdgeResponse { age: entry.age(now), entry, status: CacheStatus::Stale, coalesced: false });
                }
                Freshness::Expired => {}
            }
        }
        Metrics::inc(&self.metrics.misses);
        let (result, role) = self.fetch_coalesced(key).await;
        let coalesced = role == Role::Follower;
        if coalesced {
            Metrics::inc(&self.metrics.coalesced);
        }
        let entry = result?;
        Ok(EdgeResponse { age: entry.age(Instant::now()), entry, status: CacheStatus::Miss, coalesced })
    }

    fn revalidate_in_background(self: &Arc<Self>, key: &str) {
        let claimed = self
            .revalidating
            .lock()
            .map(|mut set| set.insert(key.to_string()))
            .unwrap_or(false);
        if !claimed {
            return;
        }
        Metrics::inc(&self.metrics.revalidations);
        let edge = Arc::clone(self);
        let key = key.to_string();
        tokio::spawn(async move {
            let _ = edge.fetch_coalesced(&key).await;
            if let Ok(mut set) = edge.revalidating.lock() {
                set.remove(&key);
            }
        });
    }

    async fn fetch_coalesced(self: &Arc<Self>, key: &str) -> (Result<Entry, CdnError>, Role) {
        let edge = Arc::clone(self);
        let owned = key.to_string();
        self.coalescer
            .run(key, move || async move { edge.fetch_and_store(owned).await })
            .await
    }

    async fn fetch_and_store(&self, key: String) -> Result<Entry, CdnError> {
        Metrics::inc(&self.metrics.origin_fetches);
        let response = self.origin.fetch(&key).await.inspect_err(|_| {
            Metrics::inc(&self.metrics.origin_errors);
        })?;
        let policy = CachePolicy::parse(response.cache_control.as_deref());
        let entry = Entry {
            body: response.body,
            content_type: response.content_type,
            policy,
            stored_at: Instant::now(),
        };
        if policy.cacheable() {
            let evicted = self
                .cache
                .lock()
                .map(|mut c| c.put(key, entry.clone()).len())
                .unwrap_or(0);
            Metrics::add(&self.metrics.evictions, evicted as u64);
        }
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::LruCache;
    use crate::origin_client::OriginResponse;
    use bytes::Bytes;
    use std::future::Future;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct MockOrigin {
        fetches: Arc<AtomicU64>,
        latency: Duration,
        cache_control: Option<String>,
        fail: bool,
    }

    impl MockOrigin {
        fn new(latency_ms: u64, cache_control: Option<&str>) -> (Self, Arc<AtomicU64>) {
            let fetches = Arc::new(AtomicU64::new(0));
            let origin = MockOrigin {
                fetches: fetches.clone(),
                latency: Duration::from_millis(latency_ms),
                cache_control: cache_control.map(str::to_string),
                fail: false,
            };
            (origin, fetches)
        }
    }

    impl OriginClient for MockOrigin {
        fn fetch(&self, key: &str) -> impl Future<Output = Result<OriginResponse, CdnError>> + Send {
            let version = self.fetches.fetch_add(1, Ordering::SeqCst);
            let latency = self.latency;
            let cache_control = self.cache_control.clone();
            let fail = self.fail;
            let body = Bytes::from(format!("{key}-v{version}"));
            async move {
                tokio::time::sleep(latency).await;
                if fail {
                    return Err(CdnError::OriginStatus(503));
                }
                Ok(OriginResponse { body, content_type: "video/mp2t".into(), cache_control })
            }
        }
    }

    const POLICY: &str = "public, max-age=10, stale-while-revalidate=30";

    fn edge(origin: MockOrigin, capacity: usize) -> Arc<Edge<LruCache, MockOrigin>> {
        Edge::new(LruCache::new(capacity), origin)
    }

    async fn settle() {
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    #[tokio::test(start_paused = true)]
    async fn hundred_concurrent_misses_for_one_key_cost_exactly_one_origin_fetch() {
        let (origin, fetches) = MockOrigin::new(50, Some(POLICY));
        let edge = edge(origin, 1 << 20);
        let tasks: Vec<_> = (0..100)
            .map(|_| {
                let edge = edge.clone();
                tokio::spawn(async move { edge.get("seg-1.ts").await })
            })
            .collect();
        let mut bodies = HashSet::new();
        for task in tasks {
            let response = task.await.unwrap().unwrap();
            assert_eq!(response.status, CacheStatus::Miss);
            bodies.insert(response.entry.body);
        }
        assert_eq!(fetches.load(Ordering::SeqCst), 1);
        assert_eq!(bodies.len(), 1);
        let m = edge.metrics();
        assert_eq!(m.misses, 100);
        assert_eq!(m.coalesced, 99);
        assert_eq!(m.origin_fetches, 1);
        assert_eq!(m.in_flight, 0);
    }

    #[tokio::test(start_paused = true)]
    async fn coalescing_is_per_key_so_distinct_objects_are_not_merged() {
        let (origin, fetches) = MockOrigin::new(50, Some(POLICY));
        let edge = edge(origin, 1 << 20);
        let tasks: Vec<_> = (0..100)
            .map(|i| {
                let edge = edge.clone();
                tokio::spawn(async move { edge.get(&format!("seg-{}.ts", i % 10)).await })
            })
            .collect();
        for task in tasks {
            let response = task.await.unwrap().unwrap();
            assert!(response.entry.body.starts_with(b"seg-"));
        }
        assert_eq!(fetches.load(Ordering::SeqCst), 10);
        assert_eq!(edge.metrics().coalesced, 90);
    }

    #[tokio::test(start_paused = true)]
    async fn origin_failure_is_shared_by_followers_and_not_cached() {
        let (mut origin, fetches) = MockOrigin::new(50, Some(POLICY));
        origin.fail = true;
        let edge = edge(origin, 1 << 20);
        let tasks: Vec<_> = (0..20)
            .map(|_| {
                let edge = edge.clone();
                tokio::spawn(async move { edge.get("seg-1.ts").await })
            })
            .collect();
        for task in tasks {
            assert_eq!(task.await.unwrap().unwrap_err(), CdnError::OriginStatus(503));
        }
        assert_eq!(fetches.load(Ordering::SeqCst), 1);
        assert_eq!(edge.metrics().cache_entries, 0);
        assert_eq!(edge.metrics().origin_errors, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn fresh_entry_is_a_hit_without_touching_origin() {
        let (origin, fetches) = MockOrigin::new(10, Some(POLICY));
        let edge = edge(origin, 1 << 20);
        assert_eq!(edge.get("a").await.unwrap().status, CacheStatus::Miss);
        tokio::time::advance(Duration::from_secs(9)).await;
        let response = edge.get("a").await.unwrap();
        assert_eq!(response.status, CacheStatus::Hit);
        assert!(response.age >= Duration::from_secs(9));
        assert_eq!(fetches.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn stale_within_window_is_served_instantly_and_refreshed_in_background() {
        let (origin, fetches) = MockOrigin::new(500, Some(POLICY));
        let edge = edge(origin, 1 << 20);
        edge.get("a").await.unwrap();
        tokio::time::advance(Duration::from_secs(11)).await;

        let started = Instant::now();
        let stale = edge.get("a").await.unwrap();
        assert_eq!(stale.status, CacheStatus::Stale);
        assert_eq!(stale.entry.body, Bytes::from("a-v0"));
        assert_eq!(Instant::now() - started, Duration::ZERO);

        settle().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert_eq!(fetches.load(Ordering::SeqCst), 2);

        let refreshed = edge.get("a").await.unwrap();
        assert_eq!(refreshed.status, CacheStatus::Hit);
        assert_eq!(refreshed.entry.body, Bytes::from("a-v1"));
        assert_eq!(edge.metrics().revalidations, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn many_stale_reads_trigger_a_single_background_refresh() {
        let (origin, fetches) = MockOrigin::new(500, Some(POLICY));
        let edge = edge(origin, 1 << 20);
        edge.get("a").await.unwrap();
        tokio::time::advance(Duration::from_secs(11)).await;
        for _ in 0..50 {
            assert_eq!(edge.get("a").await.unwrap().status, CacheStatus::Stale);
        }
        settle().await;
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert_eq!(fetches.load(Ordering::SeqCst), 2);
        assert_eq!(edge.metrics().stale, 50);
        assert_eq!(edge.metrics().revalidations, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn past_the_swr_window_the_client_waits_for_a_fresh_copy() {
        let (origin, fetches) = MockOrigin::new(100, Some(POLICY));
        let edge = edge(origin, 1 << 20);
        edge.get("a").await.unwrap();
        tokio::time::advance(Duration::from_secs(41)).await;
        let response = edge.get("a").await.unwrap();
        assert_eq!(response.status, CacheStatus::Miss);
        assert_eq!(response.entry.body, Bytes::from("a-v1"));
        assert_eq!(fetches.load(Ordering::SeqCst), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn no_store_responses_always_go_to_origin() {
        let (origin, fetches) = MockOrigin::new(10, Some("no-store"));
        let edge = edge(origin, 1 << 20);
        edge.get("a").await.unwrap();
        assert_eq!(edge.get("a").await.unwrap().status, CacheStatus::Miss);
        assert_eq!(fetches.load(Ordering::SeqCst), 2);
        assert_eq!(edge.metrics().cache_entries, 0);
    }

    #[tokio::test(start_paused = true)]
    async fn byte_budget_evicts_cold_keys_and_forces_refetch() {
        let (origin, fetches) = MockOrigin::new(10, Some(POLICY));
        let one_entry = "k0-v0".len() + "video/mp2t".len();
        let edge = edge(origin, one_entry * 2);
        edge.get("k0").await.unwrap();
        edge.get("k1").await.unwrap();
        edge.get("k0").await.unwrap();
        edge.get("k2").await.unwrap();
        assert_eq!(edge.metrics().evictions, 1);
        assert_eq!(edge.get("k0").await.unwrap().status, CacheStatus::Hit);
        assert_eq!(edge.get("k1").await.unwrap().status, CacheStatus::Miss);
        assert_eq!(fetches.load(Ordering::SeqCst), 4);
    }
}
