use herd::admission::max_in_window;
use herd::sim::report::per_second;
use herd::sim::{SimConfig, SimReport, simulate, strategy_by_name};

fn launch() -> SimConfig {
    SimConfig {
        users: 40_000,
        api_rps: 1_000,
        origin_rps: 200,
        admission_rps: 800,
        edges: 10,
        ..SimConfig::default()
    }
}

fn run(name: &str, cfg: &SimConfig) -> SimReport {
    let strategy = strategy_by_name(name, cfg).unwrap();
    simulate(strategy.as_ref(), cfg).unwrap()
}

#[test]
fn waiting_room_never_admits_more_than_the_sustainable_rate() {
    let cfg = launch();
    for name in ["waiting-room", "combined"] {
        let r = run(name, &cfg);
        assert!(r.max_admitted_per_s <= cfg.admission_rps, "{name} admitted {} in one second", r.max_admitted_per_s);
        assert!(per_second(&r.series).iter().all(|b| b.admitted <= cfg.admission_rps));
        let per_bucket = cfg.admission_rps * r.bucket_ms / 1_000 + 1;
        assert!(r.series.iter().all(|b| b.admitted <= per_bucket), "admissions must be smooth, not bursty inside a second");
    }
}

#[test]
fn every_viewer_either_plays_or_gives_up_and_none_is_lost() {
    let cfg = launch();
    for name in ["naive", "jittered-backoff", "cdn-prewarm", "waiting-room", "combined"] {
        let r = run(name, &cfg);
        assert_eq!(r.succeeded + r.abandoned, r.users, "{name} lost viewers");
        assert_eq!(r.series.iter().map(|b| b.first_frames).sum::<u64>(), r.succeeded);
    }
}

#[test]
fn naive_launch_floods_the_api_far_beyond_capacity_and_most_viewers_fail() {
    let cfg = launch();
    let r = run("naive", &cfg);
    assert!(r.peak_api_rps > 10 * cfg.api_rps);
    assert!(r.success_rate < 0.5);
    assert!(r.wasted_retries > u64::from(cfg.users));
}

#[test]
fn jittered_backoff_converts_wasted_retries_into_successful_plays() {
    let cfg = launch();
    let naive = run("naive", &cfg);
    let backoff = run("jittered-backoff", &cfg);
    assert!(backoff.success_rate > naive.success_rate + 0.3);
    assert!(backoff.peak_api_rps < naive.peak_api_rps);
}

#[test]
fn prewarming_keeps_the_origin_out_of_the_launch_entirely() {
    let cfg = launch();
    let cold = run("naive", &cfg);
    let warm = run("cdn-prewarm", &cfg);
    assert!(cold.peak_origin_rps > cfg.origin_rps, "cold cache must stampede the origin");
    assert_eq!(warm.peak_origin_rps, 0);
    assert_eq!(warm.prewarmed_objects, u64::from(cfg.edges * cfg.objects_per_edge()));
}

#[test]
fn waiting_room_alone_still_stampedes_a_cold_origin() {
    let cfg = launch();
    let r = run("waiting-room", &cfg);
    assert!(r.peak_origin_rps > 0);
    assert!(r.peak_api_rps <= cfg.api_rps + cfg.api_rps / 20);
}

#[test]
fn combined_plays_for_everyone_without_touching_origin_or_retrying() {
    let cfg = launch();
    let r = run("combined", &cfg);
    assert_eq!(r.succeeded, r.users);
    assert_eq!(r.peak_origin_rps, 0);
    assert_eq!(r.wasted_retries, 0);
    let p99 = r.ttff_p99_ms.unwrap();
    let drain_ms = u64::from(cfg.users) * 1_000 / cfg.admission_rps;
    assert!(p99 <= drain_ms + 1_000, "p99 {p99} must be bounded by queue drain time {drain_ms}");
}

#[test]
fn same_seed_reproduces_the_exact_launch_and_another_seed_does_not() {
    let cfg = launch();
    let a = run("jittered-backoff", &cfg);
    let b = run("jittered-backoff", &cfg);
    let c = run("jittered-backoff", &SimConfig { seed: 7, ..cfg });
    assert_eq!(a, b);
    assert_ne!(a.series, c.series);
}

#[test]
fn percentiles_are_ordered() {
    let r = run("waiting-room", &launch());
    assert!(r.ttff_p50_ms <= r.ttff_p95_ms && r.ttff_p95_ms <= r.ttff_p99_ms);
}

#[test]
fn sliding_window_counter_catches_bursts_split_across_second_boundaries() {
    let times = [900_000, 950_000, 1_000_000, 1_050_000];
    assert_eq!(max_in_window(&times, 1_000_000), 4);
}
