mod common;

use common::fake_pipeline;
use per_title::store::{AnalysisStore, JsonFileStore};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn every_title_gets_every_grid_point_measured_and_progress_reaches_the_total() {
    let pipeline = fake_pipeline("grid", false);
    let last = AtomicUsize::new(0);
    let analysis = pipeline
        .run(&|done, _| {
            last.fetch_max(done, Ordering::SeqCst);
        })
        .expect("analysis");
    assert_eq!(analysis.titles.len(), 2);
    for t in &analysis.titles {
        assert_eq!(t.points.len(), 6);
        assert!(!t.hulls["vmaf"].is_empty());
        assert!(!t.ladder.is_empty());
    }
    assert_eq!(last.load(Ordering::SeqCst), pipeline.total_jobs());
}

#[test]
fn bitrate_comes_from_the_encoded_size_not_the_requested_budget() {
    let analysis = fake_pipeline("size", false).run(&|_, _| {}).expect("analysis");
    let p = &analysis.titles[0].points[0];
    assert_eq!(p.target_kbps, 100);
    assert!((p.bitrate_kbps - 100.0).abs() < 1.0, "2s of 2x100kbit bytes is 100 kbps, got {}", p.bitrate_kbps);
}

#[test]
fn a_failed_encode_fails_the_whole_analysis_instead_of_hiding_a_gap() {
    assert!(fake_pipeline("fail", true).run(&|_, _| {}).is_err());
}

#[test]
fn ladder_metric_that_is_not_measured_is_rejected() {
    let mut pipeline = fake_pipeline("metric", false);
    pipeline.config.ladder_metric = "psnr".into();
    assert!(pipeline.run(&|_, _| {}).is_err());
}

#[test]
fn cached_json_round_trips_so_a_restart_serves_the_same_results() {
    let pipeline = fake_pipeline("store", false);
    let analysis = pipeline.run(&|_, _| {}).expect("analysis");
    let store = JsonFileStore {
        path: pipeline.work_dir.join("analysis.json"),
    };
    store.save(&analysis).expect("save");
    assert_eq!(store.load().expect("load"), Some(analysis));
}
