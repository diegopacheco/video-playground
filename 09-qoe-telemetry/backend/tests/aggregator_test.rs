use qoe::aggregator::{Aggregator, Dimension, TumblingWindows, WindowResult, percentile};
use qoe::beacon::{Beacon, BeaconKind};

const W: i64 = 10_000;
const BASE: i64 = 1_700_000_000_000;

fn beacon(session: &str, cdn: &str, ts: i64, event: BeaconKind) -> Beacon {
    Beacon {
        session_id: session.into(),
        title: "big-heist".into(),
        cdn: cdn.into(),
        ts,
        event,
    }
}

fn find<'a>(results: &'a [WindowResult], dim: Dimension, key: &str) -> &'a WindowResult {
    results
        .iter()
        .find(|r| r.dimension == dim && r.key == key)
        .expect("window result present")
}

#[test]
fn window_boundary_is_start_inclusive_end_exclusive_so_events_are_never_double_counted() {
    let mut agg = TumblingWindows::new(W);
    agg.ingest(&beacon("s1", "a", BASE + W - 1, BeaconKind::Startup { startup_ms: 100 }));
    agg.ingest(&beacon("s2", "a", BASE + W, BeaconKind::Startup { startup_ms: 900 }));
    let results = agg.advance(BASE + 2 * W);
    let cdn: Vec<_> = results.iter().filter(|r| r.dimension == Dimension::Cdn).collect();
    assert_eq!(cdn.len(), 2);
    assert_eq!(cdn[0].window_start, BASE);
    assert_eq!(cdn[0].metrics.avg_startup_ms, 100.0);
    assert_eq!(cdn[1].window_start, BASE + W);
    assert_eq!(cdn[1].metrics.avg_startup_ms, 900.0);
}

#[test]
fn windows_stay_open_until_watermark_passes_their_end_so_late_arrivals_still_count() {
    let mut agg = TumblingWindows::new(W);
    agg.ingest(&beacon("s1", "a", BASE + 1_000, BeaconKind::Heartbeat { played_ms: 1_000 }));
    assert!(agg.advance(BASE + W - 1).is_empty());
    assert!(agg.ingest(&beacon("s1", "a", BASE + 9_000, BeaconKind::Heartbeat { played_ms: 1_000 })));
    let results = agg.advance(BASE + W);
    assert_eq!(find(&results, Dimension::Cdn, "a").metrics.played_ms, 2_000);
}

#[test]
fn events_for_already_emitted_windows_are_dropped_and_counted_not_silently_merged() {
    let mut agg = TumblingWindows::new(W);
    agg.ingest(&beacon("s1", "a", BASE, BeaconKind::Heartbeat { played_ms: 1_000 }));
    let first = agg.advance(BASE + W);
    assert_eq!(first.len(), 2);
    assert!(!agg.ingest(&beacon("s1", "a", BASE + 5, BeaconKind::Heartbeat { played_ms: 1_000 })));
    assert_eq!(agg.late_dropped(), 1);
    assert!(agg.advance(BASE + 5 * W).is_empty());
}

#[test]
fn watermark_never_moves_backwards_so_closed_windows_cannot_reopen() {
    let mut agg = TumblingWindows::new(W);
    agg.advance(BASE + W);
    agg.advance(BASE);
    assert!(!agg.ingest(&beacon("s1", "a", BASE + 1, BeaconKind::RebufferStart)));
}

#[test]
fn rebuffer_ratio_is_stall_time_over_total_watch_time() {
    let mut agg = TumblingWindows::new(W);
    agg.ingest(&beacon("s1", "a", BASE, BeaconKind::Heartbeat { played_ms: 3_000 }));
    agg.ingest(&beacon("s1", "a", BASE + 10, BeaconKind::RebufferStart));
    agg.ingest(&beacon("s1", "a", BASE + 20, BeaconKind::RebufferEnd { rebuffer_ms: 1_000 }));
    let results = agg.advance(BASE + W);
    let m = &find(&results, Dimension::Cdn, "a").metrics;
    assert_eq!(m.rebuffer_ms, 1_000);
    assert!((m.rebuffer_ratio - 0.25).abs() < 1e-9);
}

#[test]
fn switches_per_minute_normalises_by_played_time_so_busy_and_quiet_windows_compare() {
    let mut agg = TumblingWindows::new(W);
    agg.ingest(&beacon("s1", "a", BASE, BeaconKind::Heartbeat { played_ms: 30_000 }));
    for i in 0..3 {
        agg.ingest(&beacon("s1", "a", BASE + i, BeaconKind::BitrateSwitch { from_kbps: 800, to_kbps: 1_800 }));
    }
    let results = agg.advance(BASE + W);
    assert!((find(&results, Dimension::Cdn, "a").metrics.switches_per_min - 6.0).abs() < 1e-9);
}

#[test]
fn error_rate_counts_sessions_with_errors_not_raw_error_events() {
    let mut agg = TumblingWindows::new(W);
    for s in ["s1", "s2", "s3", "s4"] {
        agg.ingest(&beacon(s, "a", BASE, BeaconKind::Heartbeat { played_ms: 1_000 }));
    }
    for _ in 0..5 {
        agg.ingest(&beacon("s1", "a", BASE + 1, BeaconKind::Error { code: "E".into() }));
    }
    let results = agg.advance(BASE + W);
    let m = &find(&results, Dimension::Cdn, "a").metrics;
    assert_eq!(m.sessions, 4);
    assert!((m.error_rate - 0.25).abs() < 1e-9);
}

#[test]
fn p95_exposes_the_slow_tail_that_the_average_hides() {
    let mut agg = TumblingWindows::new(W);
    for i in 0..19 {
        agg.ingest(&beacon(&format!("s{i}"), "a", BASE, BeaconKind::Startup { startup_ms: 500 }));
    }
    agg.ingest(&beacon("slow", "a", BASE, BeaconKind::Startup { startup_ms: 500 }));
    agg.ingest(&beacon("slower", "a", BASE, BeaconKind::Startup { startup_ms: 9_000 }));
    let results = agg.advance(BASE + W);
    let m = &find(&results, Dimension::Cdn, "a").metrics;
    assert!(m.avg_startup_ms < 1_000.0);
    assert_eq!(percentile(&[500; 19], 0.95), 500.0);
    assert_eq!(m.p95_startup_ms, 500.0);
    let mut tail = vec![500u64; 18];
    tail.extend([9_000, 9_000]);
    assert_eq!(percentile(&tail, 0.95), 9_000.0);
}

#[test]
fn every_beacon_feeds_both_cdn_and_title_views_so_a_bad_cdn_is_isolated() {
    let mut agg = TumblingWindows::new(W);
    agg.ingest(&beacon("good", "cdn-alpha", BASE, BeaconKind::Startup { startup_ms: 600 }));
    agg.ingest(&beacon("bad", "cdn-gamma", BASE, BeaconKind::Startup { startup_ms: 6_000 }));
    let results = agg.advance(BASE + W);
    assert_eq!(results.len(), 3);
    assert_eq!(find(&results, Dimension::Cdn, "cdn-alpha").metrics.avg_startup_ms, 600.0);
    assert_eq!(find(&results, Dimension::Cdn, "cdn-gamma").metrics.avg_startup_ms, 6_000.0);
    assert_eq!(find(&results, Dimension::Title, "big-heist").metrics.sessions, 2);
}

#[test]
fn empty_windows_report_zero_metrics_instead_of_nan() {
    let mut agg = TumblingWindows::new(W);
    agg.ingest(&beacon("s1", "a", BASE, BeaconKind::RebufferStart));
    let results = agg.advance(BASE + W);
    let m = &find(&results, Dimension::Cdn, "a").metrics;
    assert_eq!(m.startups, 0);
    assert_eq!(m.avg_startup_ms, 0.0);
    assert_eq!(m.rebuffer_ratio, 0.0);
    assert_eq!(m.switches_per_min, 0.0);
    assert!(serde_json::to_string(m).is_ok());
}

#[test]
fn negative_timestamps_align_to_window_floor() {
    let agg = TumblingWindows::new(W);
    assert_eq!(agg.window_start(-1), -W);
    assert_eq!(agg.window_start(BASE + 12_345), BASE + W);
}
