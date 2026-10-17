use per_title::hull::{Axis, bitrate_for, convex_hull, pareto_front};
use per_title::model::{Point, Resolution};
use std::collections::BTreeMap;

const VMAF: Axis = Axis {
    metric: "vmaf",
    min_gain: 0.5,
};

fn p(height: u32, kbps: f64, vmaf: f64) -> Point {
    Point {
        resolution: Resolution::new(height * 16 / 9, height),
        target_kbps: kbps as u32,
        bitrate_kbps: kbps,
        scores: BTreeMap::from([("vmaf".to_string(), vmaf)]),
    }
}

#[test]
fn a_point_costing_more_for_less_quality_never_enters_the_hull() {
    let points = vec![p(180, 100.0, 60.0), p(360, 200.0, 55.0), p(360, 400.0, 80.0)];
    let hull = convex_hull(&points, VMAF);
    assert!(!hull.contains(&points[1]), "dominated encode would waste bits");
    assert_eq!(hull.len(), 2);
}

#[test]
fn a_point_below_the_chord_of_its_neighbours_is_dropped_so_the_hull_stays_concave() {
    let points = vec![p(180, 100.0, 50.0), p(270, 200.0, 55.0), p(360, 400.0, 80.0)];
    let hull = convex_hull(&points, VMAF);
    assert_eq!(hull, vec![points[0].clone(), points[2].clone()]);
}

#[test]
fn hull_switches_resolution_where_the_higher_resolution_starts_winning() {
    let points = vec![
        p(180, 100.0, 60.0),
        p(180, 400.0, 70.0),
        p(180, 1600.0, 72.0),
        p(360, 100.0, 40.0),
        p(360, 400.0, 78.0),
        p(360, 1600.0, 92.0),
    ];
    let hull = convex_hull(&points, VMAF);
    let heights: Vec<u32> = hull.iter().map(|h| h.resolution.height).collect();
    assert_eq!(heights, vec![180, 360, 360]);
}

#[test]
fn hull_is_strictly_increasing_in_bitrate_and_quality() {
    let points = vec![
        p(180, 60.0, 40.0),
        p(180, 100.0, 55.0),
        p(270, 160.0, 66.0),
        p(270, 250.0, 74.0),
        p(360, 400.0, 82.0),
        p(360, 650.0, 88.0),
    ];
    let hull = convex_hull(&points, VMAF);
    assert!(hull.windows(2).all(|w| w[1].bitrate_kbps > w[0].bitrate_kbps));
    assert!(hull.windows(2).all(|w| w[1].score("vmaf") > w[0].score("vmaf")));
}

#[test]
fn saturated_quality_gains_below_the_noticeable_threshold_do_not_buy_more_bits() {
    let points = vec![p(360, 100.0, 99.8), p(360, 1600.0, 100.0), p(360, 4000.0, 100.0)];
    assert_eq!(pareto_front(&points, VMAF).len(), 1);
}

#[test]
fn bitrate_for_interpolates_on_a_log_scale_between_hull_points() {
    let hull = vec![p(360, 100.0, 50.0), p(360, 400.0, 70.0)];
    let k = bitrate_for(&hull, VMAF, 60.0).unwrap_or_default();
    assert!((k - 200.0).abs() < 0.01, "midpoint in log space of 100 and 400 is 200, got {k}");
}

#[test]
fn bitrate_for_is_unknown_when_quality_is_out_of_reach() {
    let hull = vec![p(360, 100.0, 50.0), p(360, 400.0, 70.0)];
    assert_eq!(bitrate_for(&hull, VMAF, 85.0), None);
    assert_eq!(bitrate_for(&hull, VMAF, 70.3), Some(400.0));
    assert_eq!(bitrate_for(&hull, VMAF, 10.0), Some(100.0));
}
