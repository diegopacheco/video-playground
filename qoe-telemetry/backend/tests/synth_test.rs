use qoe::aggregator::{Aggregator, Dimension, TumblingWindows};
use qoe::beacon::BeaconKind;
use qoe::synth::Fleet;

const BASE: i64 = 1_700_000_000_000;

#[test]
fn bad_cdn_stands_out_after_aggregation_so_the_dashboard_can_spot_it() {
    let mut fleet = Fleet::new(300, 42);
    let mut agg = TumblingWindows::new(10_000);
    for sec in 0..10 {
        for b in fleet.tick(BASE + sec * 1_000) {
            agg.ingest(&b);
        }
    }
    let results = agg.advance(BASE + 10_000);
    let metric = |key: &str| {
        results
            .iter()
            .find(|r| r.dimension == Dimension::Cdn && r.key == key)
            .map(|r| r.metrics.clone())
            .expect("cdn present")
    };
    let good = metric("cdn-alpha");
    let bad = metric("cdn-gamma");
    assert!(bad.avg_startup_ms > 2.0 * good.avg_startup_ms);
    assert!(bad.rebuffer_ratio > 3.0 * good.rebuffer_ratio);
    assert!(bad.switches_per_min > good.switches_per_min);
}

#[test]
fn every_synthetic_session_starts_with_a_startup_beacon() {
    let mut fleet = Fleet::new(20, 7);
    let first = fleet.tick(BASE);
    assert_eq!(first.len(), 20);
    assert!(first.iter().all(|b| matches!(b.event, BeaconKind::Startup { .. })));
}

#[test]
fn heartbeat_played_time_excludes_stalls_so_rebuffer_ratio_is_not_double_counted() {
    let mut fleet = Fleet::new(200, 9);
    fleet.tick(BASE);
    let beacons = fleet.tick(BASE + 1_000);
    for b in &beacons {
        if let BeaconKind::RebufferEnd { rebuffer_ms } = b.event {
            let played = beacons.iter().find_map(|h| match h.event {
                BeaconKind::Heartbeat { played_ms } if h.session_id == b.session_id => Some(played_ms),
                _ => None,
            });
            assert_eq!(played, Some(1_000 - rebuffer_ms));
        }
    }
}
