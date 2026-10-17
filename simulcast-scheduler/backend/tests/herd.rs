mod common;

use chrono::Duration;
use common::{release, rig};
use simulcast_scheduler::herd::bucket::TokenBucket;
use simulcast_scheduler::herd::jitter::Jitter;
use simulcast_scheduler::service::{Decision, HerdConfig, PlayOutcome};
use std::collections::HashSet;

#[test]
fn bucket_caps_bursts_at_capacity_and_refills_over_time() {
    let bucket = TokenBucket::new(3.0, 10.0, 0);
    assert!((0..3).all(|_| bucket.try_acquire(0)));
    assert!(!bucket.try_acquire(0));
    assert!(bucket.try_acquire(100));
    assert!(!bucket.try_acquire(100));
}

#[test]
fn bucket_ignores_clock_going_backwards_instead_of_minting_tokens() {
    let bucket = TokenBucket::new(1.0, 10.0, 1_000);
    assert!(bucket.try_acquire(1_000));
    assert!(!bucket.try_acquire(0));
    assert!(!bucket.try_acquire(1_050));
}

#[test]
fn jitter_spreads_values_within_bounds_so_clients_desynchronize() {
    let jitter = Jitter::seeded(7);
    let values: Vec<u64> = (0..500).map(|_| jitter.spread(5_000, 20)).collect();
    assert!(values.iter().all(|v| (4_000..=6_000).contains(v)));
    assert!(values.iter().collect::<HashSet<_>>().len() > 100);
}

#[test]
fn cache_hits_bypass_admission_so_the_bucket_only_guards_origin_work() {
    let config = HerdConfig { bucket_capacity: 1.0, bucket_refill_per_sec: 0.0, ..HerdConfig::default() };
    let r = rig(release() + Duration::minutes(1), config);
    for _ in 0..100 {
        assert!(matches!(r.scheduler.entitlement("ep", "US").unwrap(), Decision::Served { .. }));
    }
    let m = r.scheduler.metrics();
    assert_eq!((m.cache_misses, m.cache_hits, m.throttled), (1, 99, 0));
}

#[test]
fn stampede_beyond_capacity_gets_429_with_jittered_retry_after() {
    let config = HerdConfig { bucket_capacity: 2.0, bucket_refill_per_sec: 0.0, ..HerdConfig::default() };
    let r = rig(release() + Duration::minutes(1), config);
    let mut granted = 0;
    let mut retries = HashSet::new();
    for _ in 0..50 {
        match r.scheduler.play("ep", "US").unwrap() {
            PlayOutcome::Granted { .. } => granted += 1,
            PlayOutcome::Throttled { retry_after_ms } => {
                assert!((config.retry_base_ms..=config.retry_base_ms + config.retry_jitter_ms).contains(&retry_after_ms));
                retries.insert(retry_after_ms);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    assert_eq!(granted, 1);
    assert!(retries.len() > 10);
}

#[test]
fn cache_ttl_is_jittered_so_entries_do_not_expire_in_lockstep() {
    let config = HerdConfig { ttl_ms: 1_000, ttl_jitter_pct: 50, ..HerdConfig::default() };
    let r = rig(release() + Duration::minutes(1), config);
    for c in ["US", "BR", "GB", "DE", "JP"] {
        r.scheduler.entitlement("ep", c).unwrap();
    }
    r.clock.advance(Duration::milliseconds(1_000));
    r.scheduler.reset_metrics();
    for c in ["US", "BR", "GB", "DE", "JP"] {
        r.scheduler.entitlement("ep", c).unwrap();
    }
    let m = r.scheduler.metrics();
    assert!(m.cache_hits > 0 && m.cache_misses > 0, "hits {} misses {}", m.cache_hits, m.cache_misses);
}
