use abr::algorithms::catalog;
use abr::player::PlayerConfig;
use abr::report::run;
use abr::trace::bundled;

fn all_ids() -> Vec<String> {
    catalog().iter().map(|i| i.id.to_string()).collect()
}

#[test]
fn wall_clock_equals_startup_plus_content_plus_stalls_for_every_trace_and_algorithm() {
    let config = PlayerConfig::default();
    for trace in bundled().unwrap() {
        for r in run(&config, &trace, &all_ids()).unwrap().runs {
            let s = &r.session;
            let expected = s.startup_delay_s + s.content_s + s.rebuffer_s;
            assert!(
                (s.end_s - expected).abs() < 1e-6,
                "{} on {}: session lasted {} but startup+content+stall = {}",
                r.algorithm,
                trace.id,
                s.end_s,
                expected
            );
            let stalls: f64 = s.segments.iter().map(|x| x.stall_s).sum();
            assert!((stalls - s.rebuffer_s).abs() < 1e-6, "per segment stalls must add up to the total");
        }
    }
}

#[test]
fn stable_wifi_plays_without_any_stall_for_every_algorithm() {
    let trace = bundled().unwrap().into_iter().find(|t| t.id == "wifi-stable").unwrap();
    for r in run(&PlayerConfig::default(), &trace, &all_ids()).unwrap().runs {
        assert_eq!(r.metrics.rebuffer_events, 0, "{} stalled on a 6.5 Mbps link with a 4.3 Mbps top rung", r.algorithm);
    }
}

#[test]
fn dynamic_ramps_up_faster_than_pure_bola_at_startup() {
    let trace = bundled().unwrap().into_iter().find(|t| t.id == "wifi-stable").unwrap();
    let report = run(&PlayerConfig::default(), &trace, &["bola".into(), "dynamic".into()]).unwrap();
    let first_five = |i: usize| report.runs[i].session.segments[..5].iter().map(|s| s.bitrate_kbps).sum::<f64>();
    assert!(first_five(1) > first_five(0), "throughput at startup should beat BOLA's buffer-starved lowest picks");
}

#[test]
fn every_run_downloads_every_segment_once() {
    let config = PlayerConfig::default();
    for trace in bundled().unwrap() {
        for r in run(&config, &trace, &all_ids()).unwrap().runs {
            assert_eq!(r.session.segments.len(), config.segments);
            assert!(r.session.segments.windows(2).all(|w| w[1].request_s >= w[0].done_s - 1e-9), "downloads are sequential");
        }
    }
}
