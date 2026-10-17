mod common;

use chrono::Duration;
use common::utc;
use simulcast_scheduler::entitlement::{Access, UnlockPlan};
use simulcast_scheduler::model::{Episode, RegionRule};
use simulcast_scheduler::tzdisplay::local_time;
use std::collections::BTreeMap;

#[test]
fn spring_forward_skips_the_missing_hour_so_viewers_never_see_a_nonexistent_time() {
    let before = local_time(utc(2026, 3, 8, 6, 59, 59), "America/New_York").unwrap();
    let after = local_time(utc(2026, 3, 8, 7, 0, 0), "America/New_York").unwrap();
    assert_eq!(before.local, "2026-03-08 01:59:59");
    assert_eq!(before.abbrev, "EST");
    assert_eq!(after.local, "2026-03-08 03:00:00");
    assert_eq!(after.abbrev, "EDT");
    assert_eq!(after.utc_offset, "-04:00");
}

#[test]
fn fall_back_repeats_a_wall_clock_hour_so_the_offset_must_disambiguate_it() {
    let first = local_time(utc(2026, 11, 1, 5, 30, 0), "America/New_York").unwrap();
    let second = local_time(utc(2026, 11, 1, 6, 30, 0), "America/New_York").unwrap();
    assert_eq!(first.local, second.local);
    assert_eq!(first.local, "2026-11-01 01:30:00");
    assert_ne!(first.utc_offset, second.utc_offset);
    assert_eq!((first.abbrev.as_str(), second.abbrev.as_str()), ("EDT", "EST"));
}

#[test]
fn southern_hemisphere_dst_ends_in_april_not_november() {
    let before = local_time(utc(2026, 4, 4, 15, 59, 59), "Australia/Sydney").unwrap();
    let after = local_time(utc(2026, 4, 4, 16, 0, 0), "Australia/Sydney").unwrap();
    assert_eq!(before.local, "2026-04-05 02:59:59");
    assert_eq!(before.utc_offset, "+11:00");
    assert_eq!(after.local, "2026-04-05 02:00:00");
    assert_eq!(after.utc_offset, "+10:00");
}

#[test]
fn half_hour_zone_without_dst_keeps_a_stable_offset_year_round() {
    let winter = local_time(utc(2026, 1, 10, 0, 0, 0), "Asia/Kolkata").unwrap();
    let summer = local_time(utc(2026, 7, 10, 0, 0, 0), "Asia/Kolkata").unwrap();
    assert_eq!(winter.utc_offset, "+05:30");
    assert_eq!(summer.utc_offset, "+05:30");
}

#[test]
fn regional_delay_is_an_absolute_duration_so_it_shifts_wall_clock_across_dst() {
    let release = utc(2026, 3, 8, 1, 0, 0);
    let episode = Episode {
        id: "e".into(),
        title: "E".into(),
        release_utc: release,
        default_rule: RegionRule::open(),
        regions: BTreeMap::from([("US".into(), RegionRule::Window { delay_secs: 86_400, until: None })]),
    };
    let plan = UnlockPlan::build(&episode).unwrap();
    let Access::Window { from, .. } = plan.access("US") else { panic!("US must be windowed") };
    assert_eq!(from, release + Duration::hours(24));
    let at_release = local_time(release, "America/New_York").unwrap();
    let at_unlock = local_time(from, "America/New_York").unwrap();
    assert_eq!(at_release.local, "2026-03-07 20:00:00");
    assert_eq!(at_unlock.local, "2026-03-08 21:00:00");
}

#[test]
fn unknown_time_zone_is_an_error_not_a_silent_utc_fallback() {
    assert!(local_time(utc(2026, 1, 1, 0, 0, 0), "Mars/Olympus").is_err());
}
