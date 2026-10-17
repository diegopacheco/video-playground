use per_title::hull::{Axis, convex_hull};
use per_title::ladder::{build_ladder, compare_generic, total_savings};
use per_title::model::{Point, Rendition, Resolution};
use std::collections::BTreeMap;

const VMAF: Axis = Axis {
    metric: "vmaf",
    min_gain: 0.5,
};

fn p(height: u32, kbps: u32, vmaf: f64) -> Point {
    Point {
        resolution: Resolution::new(height * 16 / 9, height),
        target_kbps: kbps,
        bitrate_kbps: kbps as f64,
        scores: BTreeMap::from([("vmaf".to_string(), vmaf)]),
    }
}

fn rendition(height: u32, kbps: u32) -> Rendition {
    Rendition {
        resolution: Resolution::new(height * 16 / 9, height),
        kbps,
    }
}

#[test]
fn each_rung_is_the_cheapest_hull_point_that_meets_its_target() {
    let hull = vec![p(180, 100, 55.0), p(270, 250, 72.0), p(360, 600, 86.0), p(360, 1500, 94.0)];
    let ladder = build_ladder(&hull, "vmaf", &[50.0, 70.0, 85.0, 93.0]);
    let kbps: Vec<u32> = ladder.iter().map(|r| r.point.target_kbps).collect();
    assert_eq!(kbps, vec![100, 250, 600, 1500]);
    assert!(ladder.iter().all(|r| r.reached));
}

#[test]
fn easy_content_collapses_targets_onto_fewer_rungs_instead_of_repeating_one() {
    let hull = vec![p(270, 50, 78.0), p(360, 100, 99.0)];
    let ladder = build_ladder(&hull, "vmaf", &[60.0, 75.0, 85.0, 93.0]);
    assert_eq!(ladder.len(), 2);
    assert_eq!(ladder[1].point.target_kbps, 100);
}

#[test]
fn unreachable_target_is_flagged_and_uses_the_best_available_point() {
    let hull = vec![p(180, 100, 40.0), p(360, 4000, 80.0)];
    let ladder = build_ladder(&hull, "vmaf", &[30.0, 95.0]);
    assert_eq!(ladder.len(), 2);
    assert!(!ladder[1].reached);
    assert_eq!(ladder[1].point.target_kbps, 4000);
}

#[test]
fn easy_content_matches_generic_quality_for_far_fewer_bits() {
    let points = vec![p(360, 100, 99.0), p(360, 1600, 99.3), p(270, 650, 96.0), p(270, 60, 80.0)];
    let hull = convex_hull(&points, VMAF);
    let generic = compare_generic(&points, &hull, VMAF, &[rendition(270, 650), rendition(360, 1600)]);
    let savings = total_savings(&generic);
    assert!(savings.saving_pct > 80.0, "got {}", savings.saving_pct);
    assert_eq!(savings.compared_rungs, 2);
}

#[test]
fn generic_rungs_already_on_the_hull_save_nothing() {
    let points = vec![p(180, 250, 60.0), p(360, 1600, 85.0), p(270, 650, 70.0)];
    let hull = convex_hull(&points, VMAF);
    let generic = compare_generic(&points, &hull, VMAF, &[rendition(360, 1600)]);
    assert!(generic[0].saving_pct.unwrap_or(1.0).abs() < 1e-9);
}

#[test]
fn equal_quality_savings_are_never_negative_because_the_hull_dominates_every_encode() {
    let points = vec![
        p(180, 100, 40.0),
        p(180, 400, 60.0),
        p(270, 400, 65.0),
        p(360, 400, 58.0),
        p(360, 1600, 85.0),
        p(270, 1600, 80.0),
    ];
    let hull = convex_hull(&points, VMAF);
    let generic: Vec<Rendition> = points
        .iter()
        .map(|q| Rendition {
            resolution: q.resolution,
            kbps: q.target_kbps,
        })
        .collect();
    for c in compare_generic(&points, &hull, VMAF, &generic) {
        assert!(c.saving_pct.unwrap_or(0.0) >= -1e-9, "{:?}", c);
    }
}
