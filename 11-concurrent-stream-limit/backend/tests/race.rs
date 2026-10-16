use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use streamlimit::clock::{Clock, SystemClock};
use streamlimit::lease::{AcquireOutcome, AcquireRequest, LeaseStore};
use streamlimit::memory::MemoryLeaseStore;
use streamlimit::plan::Policy;
use streamlimit::redis_store::RedisLeaseStore;
use tokio::sync::Barrier;

const DEVICES: usize = 64;
const TTL: u64 = 60_000;

fn redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6111".to_string())
}

fn unique(tag: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{tag}-{}-{nanos}", std::process::id())
}

async fn redis_store() -> RedisLeaseStore {
    RedisLeaseStore::connect(&redis_url(), &unique("test"))
        .await
        .expect("redis must be running on REDIS_URL or 127.0.0.1:6111, run ./scripts/start-all.sh")
}

fn now() -> u64 {
    SystemClock.now_ms()
}

async fn acquire<S: LeaseStore>(
    store: &S,
    account: &str,
    device: &str,
    limit: usize,
    policy: Policy,
    ttl_ms: u64,
) -> AcquireOutcome {
    store
        .acquire(AcquireRequest {
            account,
            device,
            now_ms: now(),
            ttl_ms,
            limit,
            policy,
        })
        .await
        .expect("acquire must not error")
}

async fn stampede<S: LeaseStore>(store: Arc<S>, limit: usize, policy: Policy) -> (usize, usize) {
    let account = unique("stampede");
    let barrier = Arc::new(Barrier::new(DEVICES));
    let granted = Arc::new(AtomicUsize::new(0));
    let evicted = Arc::new(AtomicUsize::new(0));
    let mut tasks = Vec::new();
    for i in 0..DEVICES {
        let store = store.clone();
        let barrier = barrier.clone();
        let granted = granted.clone();
        let evicted = evicted.clone();
        let account = account.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            let device = format!("device-{i}");
            if let AcquireOutcome::Granted { evicted: e, .. } =
                acquire(store.as_ref(), &account, &device, limit, policy, TTL).await
            {
                granted.fetch_add(1, Ordering::SeqCst);
                evicted.fetch_add(e.len(), Ordering::SeqCst);
            }
        }));
    }
    for t in tasks {
        t.await.expect("task panicked");
    }
    let active = store.list(&account, now()).await.expect("list").len();
    assert!(active <= limit, "active {active} exceeded limit {limit}");
    (granted.load(Ordering::SeqCst), evicted.load(Ordering::SeqCst))
}

async fn reject_stampede_grants_exactly_the_plan_limit<S: LeaseStore>(store: Arc<S>) {
    for limit in [1, 2, 4] {
        let (granted, evicted) = stampede(store.clone(), limit, Policy::RejectNewest).await;
        assert_eq!(granted, limit, "reject_newest must grant exactly {limit} of {DEVICES}");
        assert_eq!(evicted, 0);
    }
}

async fn kick_stampede_evicts_everything_above_the_limit<S: LeaseStore>(store: Arc<S>) {
    for limit in [1, 2, 4] {
        let (granted, evicted) = stampede(store.clone(), limit, Policy::KickOldest).await;
        assert_eq!(granted, DEVICES, "kick_oldest always admits the newcomer");
        assert_eq!(evicted, DEVICES - limit, "each admission beyond {limit} kicks one stream");
    }
}

async fn churn_never_exceeds_limit<S: LeaseStore>(store: Arc<S>, policy: Policy) {
    let limit = 2;
    let account = unique("churn");
    let peak = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicUsize::new(0));
    let watcher = {
        let store = store.clone();
        let account = account.clone();
        let peak = peak.clone();
        let stop = stop.clone();
        tokio::spawn(async move {
            while stop.load(Ordering::SeqCst) == 0 {
                let n = store.list(&account, now()).await.expect("list").len();
                peak.fetch_max(n, Ordering::SeqCst);
                tokio::task::yield_now().await;
            }
        })
    };
    let mut tasks = Vec::new();
    for i in 0..24 {
        let store = store.clone();
        let account = account.clone();
        tasks.push(tokio::spawn(async move {
            let device = format!("device-{i}");
            for round in 0..41 {
                let out = acquire(store.as_ref(), &account, &device, limit, policy, TTL).await;
                if matches!(out, AcquireOutcome::Granted { .. }) && round % 3 == 0 {
                    store.release(&account, &device).await.expect("release");
                } else {
                    store
                        .heartbeat(&account, &device, now(), TTL)
                        .await
                        .expect("heartbeat");
                }
            }
        }));
    }
    for t in tasks {
        t.await.expect("task panicked");
    }
    stop.store(1, Ordering::SeqCst);
    watcher.await.expect("watcher panicked");
    let remaining = store.list(&account, now()).await.expect("list").len();
    peak.fetch_max(remaining, Ordering::SeqCst);
    let observed = peak.load(Ordering::SeqCst);
    assert!(observed <= limit, "observed {observed} concurrent leases with limit {limit}");
    assert!(observed > 0, "watcher must have seen live leases");
}

async fn crashed_device_is_recovered_by_ttl<S: LeaseStore>(store: Arc<S>) {
    let account = unique("crash");
    let ttl = 300;
    assert!(matches!(
        acquire(store.as_ref(), &account, "crashed", 1, Policy::RejectNewest, ttl).await,
        AcquireOutcome::Granted { .. }
    ));
    assert!(matches!(
        acquire(store.as_ref(), &account, "newcomer", 1, Policy::RejectNewest, ttl).await,
        AcquireOutcome::Rejected { active: 1 }
    ));
    tokio::time::sleep(Duration::from_millis(ttl + 50)).await;
    assert!(matches!(
        acquire(store.as_ref(), &account, "newcomer", 1, Policy::RejectNewest, ttl).await,
        AcquireOutcome::Granted { .. }
    ));
    assert!(
        store
            .heartbeat(&account, "crashed", now(), ttl)
            .await
            .expect("heartbeat")
            .is_none()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn memory_reject_stampede() {
    reject_stampede_grants_exactly_the_plan_limit(Arc::new(MemoryLeaseStore::new())).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn memory_kick_stampede() {
    kick_stampede_evicts_everything_above_the_limit(Arc::new(MemoryLeaseStore::new())).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn memory_churn() {
    let store = Arc::new(MemoryLeaseStore::new());
    churn_never_exceeds_limit(store.clone(), Policy::RejectNewest).await;
    churn_never_exceeds_limit(store, Policy::KickOldest).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn memory_crash_recovery() {
    crashed_device_is_recovered_by_ttl(Arc::new(MemoryLeaseStore::new())).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn redis_reject_stampede() {
    reject_stampede_grants_exactly_the_plan_limit(Arc::new(redis_store().await)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn redis_kick_stampede() {
    kick_stampede_evicts_everything_above_the_limit(Arc::new(redis_store().await)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn redis_churn() {
    let store = Arc::new(redis_store().await);
    churn_never_exceeds_limit(store.clone(), Policy::RejectNewest).await;
    churn_never_exceeds_limit(store, Policy::KickOldest).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn redis_crash_recovery() {
    crashed_device_is_recovered_by_ttl(Arc::new(redis_store().await)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn redis_zset_cardinality_never_exceeds_limit_under_parallel_connections() {
    let prefix = unique("raw");
    let limit = 4;
    let account = "hammer";
    let key = format!("{prefix}:{{{account}}}:leases");
    let mut stores = Vec::new();
    for _ in 0..8 {
        stores.push(Arc::new(
            RedisLeaseStore::connect(&redis_url(), &prefix)
                .await
                .expect("redis must be running"),
        ));
    }
    let client = redis::Client::open(redis_url()).expect("client");
    let mut raw = client
        .get_multiplexed_async_connection()
        .await
        .expect("raw connection");
    let mut tasks = Vec::new();
    for (c, store) in stores.into_iter().enumerate() {
        for d in 0..8 {
            let store = store.clone();
            tasks.push(tokio::spawn(async move {
                let device = format!("conn{c}-dev{d}");
                for round in 0..60 {
                    let out =
                        acquire(store.as_ref(), account, &device, limit, Policy::RejectNewest, TTL)
                            .await;
                    if matches!(out, AcquireOutcome::Granted { .. }) && round % 2 == 0 {
                        store.release(account, &device).await.expect("release");
                    }
                }
            }));
        }
    }
    let mut peak = 0usize;
    let mut samples = 0usize;
    while tasks.iter().any(|t| !t.is_finished()) {
        let n: usize = redis::cmd("ZCARD")
            .arg(&key)
            .query_async(&mut raw)
            .await
            .expect("zcard");
        peak = peak.max(n);
        samples += 1;
    }
    for t in tasks {
        t.await.expect("task panicked");
    }
    println!("raw zcard samples {samples} peak {peak} limit {limit}");
    assert!(peak <= limit, "raw ZCARD reached {peak} with limit {limit}");
    assert!(samples > 10, "sampler must observe the hammer while it runs");
}
