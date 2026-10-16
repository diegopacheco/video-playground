mod common;

use chrono::Duration;
use common::{release, rig};
use simulcast_scheduler::geo::{GeoResolver, HeaderThenIpResolver, UNKNOWN_COUNTRY};
use simulcast_scheduler::service::{Decision, HerdConfig, PlayOutcome};
use std::net::IpAddr;

fn ip(s: &str) -> Option<IpAddr> {
    Some(s.parse().unwrap())
}

#[test]
fn country_header_wins_over_ip_so_edge_geo_is_trusted() {
    let geo = HeaderThenIpResolver::with_default_table();
    assert_eq!(geo.resolve(Some("jp"), ip("10.1.2.3")), "JP");
}

#[test]
fn ip_range_table_resolves_when_no_header_is_present() {
    let geo = HeaderThenIpResolver::with_default_table();
    assert_eq!(geo.resolve(None, ip("10.1.200.9")), "US");
    assert_eq!(geo.resolve(None, ip("10.8.0.1")), "CN");
}

#[test]
fn malformed_header_falls_through_to_ip_instead_of_inventing_a_country() {
    let geo = HeaderThenIpResolver::with_default_table();
    assert_eq!(geo.resolve(Some("U$A"), ip("10.2.0.1")), "BR");
}

#[test]
fn unresolvable_client_is_unknown() {
    let geo = HeaderThenIpResolver::with_default_table();
    assert_eq!(geo.resolve(None, ip("192.168.1.1")), UNKNOWN_COUNTRY);
    assert_eq!(geo.resolve(None, ip("::1")), UNKNOWN_COUNTRY);
}

#[test]
fn blocked_region_is_geo_blocked_even_after_release_and_never_gets_a_countdown() {
    let r = rig(release() + Duration::hours(1), HerdConfig::default());
    let Decision::Served { view, retry_after_ms, .. } = r.scheduler.entitlement("ep", "CN").unwrap() else {
        panic!("not throttled");
    };
    assert_eq!(view.status, "geo_blocked");
    assert!(view.countdown_ms.is_none());
    assert!(retry_after_ms.is_none());
}

#[test]
fn unknown_region_is_blocked_rather_than_defaulting_to_open() {
    let r = rig(release() + Duration::hours(1), HerdConfig::default());
    let Decision::Served { view, .. } = r.scheduler.entitlement("ep", UNKNOWN_COUNTRY).unwrap() else {
        panic!("not throttled");
    };
    assert_eq!(view.status, "geo_blocked");
    assert_eq!(serde_json::to_value(view.reason).unwrap(), "unknown_region");
}

#[test]
fn expired_license_turns_an_available_region_into_geo_blocked() {
    let r = rig(release() + Duration::days(7), HerdConfig::default());
    let Decision::Served { view, .. } = r.scheduler.entitlement("ep", "KR").unwrap() else {
        panic!("not throttled");
    };
    assert_eq!(serde_json::to_value(view.reason).unwrap(), "license_expired");
}

#[test]
fn play_in_blocked_region_is_refused_without_issuing_a_session() {
    let r = rig(release() + Duration::hours(1), HerdConfig::default());
    assert!(matches!(r.scheduler.play("ep", "CN").unwrap(), PlayOutcome::Blocked { .. }));
    assert_eq!(r.scheduler.metrics().plays_granted, 0);
}
