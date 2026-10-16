mod common;

use chrono::Duration;
use common::{release, rig, utc};
use simulcast_scheduler::entitlement::{Access, BlockReason, Status, evaluate};
use simulcast_scheduler::model::RegionRule;
use simulcast_scheduler::service::{Decision, HerdConfig};

fn served(decision: Decision) -> (String, bool) {
    match decision {
        Decision::Served { view, cache_hit, .. } => (view.status.to_string(), cache_hit),
        Decision::Throttled { .. } => panic!("unexpected throttle"),
    }
}

#[test]
fn unlock_is_inclusive_at_the_exact_release_instant() {
    let access = Access::Window { from: release(), until: None };
    assert_eq!(evaluate(access, release() - Duration::milliseconds(1)), Status::Locked { unlock_at: release() });
    assert_eq!(evaluate(access, release()), Status::Available { since: release(), until: None });
}

#[test]
fn license_end_is_exclusive_so_the_last_millisecond_is_still_playable() {
    let end = release() + Duration::days(1);
    let access = Access::Window { from: release(), until: Some(end) };
    assert!(matches!(evaluate(access, end - Duration::milliseconds(1)), Status::Available { .. }));
    assert_eq!(evaluate(access, end), Status::GeoBlocked { reason: BlockReason::LicenseExpired });
}

#[test]
fn cached_locked_response_never_outlives_the_unlock_instant() {
    let r = rig(release() - Duration::milliseconds(500), HerdConfig::default());
    assert_eq!(served(r.scheduler.entitlement("ep", "US").unwrap()), ("locked".into(), false));
    r.clock.set(release());
    let (status, _) = served(r.scheduler.entitlement("ep", "US").unwrap());
    assert_eq!(status, "available");
}

#[test]
fn warmer_precomputes_the_unlock_so_the_first_request_at_the_instant_is_a_cache_hit() {
    let r = rig(release() - Duration::seconds(1), HerdConfig::default());
    assert!(r.scheduler.warm().unwrap() > 0);
    r.clock.set(release());
    assert_eq!(served(r.scheduler.entitlement("ep", "US").unwrap()), ("available".into(), true));
}

#[test]
fn warmer_does_not_leak_the_future_status_before_the_instant() {
    let r = rig(release() - Duration::seconds(1), HerdConfig::default());
    r.scheduler.warm().unwrap();
    r.clock.set(release() - Duration::milliseconds(1));
    assert_eq!(served(r.scheduler.entitlement("ep", "US").unwrap()).0, "locked");
}

#[test]
fn delayed_region_stays_locked_while_simulcast_regions_are_open() {
    let r = rig(release() + Duration::minutes(59), HerdConfig::default());
    assert_eq!(served(r.scheduler.entitlement("ep", "US").unwrap()).0, "available");
    assert_eq!(served(r.scheduler.entitlement("ep", "IN").unwrap()).0, "locked");
    r.clock.set(release() + Duration::hours(1));
    assert_eq!(served(r.scheduler.entitlement("ep", "IN").unwrap()).0, "available");
}

#[test]
fn locked_retry_after_never_tells_clients_to_come_back_before_unlock() {
    let r = rig(release() - Duration::seconds(10), HerdConfig::default());
    let Decision::Served { view, retry_after_ms: Some(retry), .. } = r.scheduler.entitlement("ep", "US").unwrap() else {
        panic!("locked response must carry retry-after");
    };
    let countdown = view.countdown_ms.unwrap() as u64;
    assert_eq!(countdown, 10_000);
    assert!(retry >= countdown && retry <= countdown + HerdConfig::default().unlock_spread_ms);
}

#[test]
fn reschedule_invalidates_cached_status_so_a_moved_release_takes_effect_now() {
    let r = rig(release() - Duration::seconds(30), HerdConfig::default());
    assert_eq!(served(r.scheduler.entitlement("ep", "US").unwrap()).0, "locked");
    r.scheduler.reschedule("ep", -5).unwrap();
    assert_eq!(served(r.scheduler.entitlement("ep", "US").unwrap()), ("available".into(), false));
}

#[test]
fn invalid_windows_are_rejected_at_schedule_time() {
    let r = rig(utc(2026, 1, 1, 0, 0, 0), HerdConfig::default());
    let mut bad = common::episode();
    bad.regions.insert("US".into(), RegionRule::Window { delay_secs: -1, until: None });
    assert!(r.scheduler.upsert(bad).is_err());
    let mut ends_early = common::episode();
    ends_early
        .regions
        .insert("US".into(), RegionRule::Window { delay_secs: 0, until: Some(release()) });
    assert!(r.scheduler.upsert(ends_early).is_err());
}
