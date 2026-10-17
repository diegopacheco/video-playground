use crate::hull::{Axis, bitrate_for};
use crate::model::{GenericComparison, LadderRung, Point, Rendition, Savings};

pub fn build_ladder(hull: &[Point], metric: &str, targets: &[f64]) -> Vec<LadderRung> {
    let mut ladder: Vec<LadderRung> = Vec::new();
    for &target in targets {
        let pick = hull
            .iter()
            .find(|p| p.score(metric) >= target)
            .map(|p| (p, true))
            .or_else(|| hull.last().map(|p| (p, false)));
        if let Some((point, reached)) = pick {
            if ladder.last().is_some_and(|r| r.point == *point) {
                continue;
            }
            ladder.push(LadderRung {
                target,
                reached,
                point: point.clone(),
            });
        }
    }
    ladder
}

pub fn compare_generic(
    points: &[Point],
    hull: &[Point],
    axis: Axis,
    generic: &[Rendition],
) -> Vec<GenericComparison> {
    generic
        .iter()
        .filter_map(|r| points.iter().find(|p| p.is(r)))
        .map(|point| {
            let equal_quality_kbps = bitrate_for(hull, axis, point.score(axis.metric));
            let saving_pct = equal_quality_kbps.map(|k| (1.0 - k / point.bitrate_kbps) * 100.0);
            GenericComparison {
                point: point.clone(),
                equal_quality_kbps,
                saving_pct,
            }
        })
        .collect()
}

pub fn total_savings(comparisons: &[GenericComparison]) -> Savings {
    let matched: Vec<(f64, f64)> = comparisons
        .iter()
        .filter_map(|c| c.equal_quality_kbps.map(|k| (c.point.bitrate_kbps, k)))
        .collect();
    let generic_kbps: f64 = matched.iter().map(|(g, _)| g).sum();
    let per_title_kbps: f64 = matched.iter().map(|(_, p)| p).sum();
    let saving_pct = if generic_kbps > 0.0 {
        (1.0 - per_title_kbps / generic_kbps) * 100.0
    } else {
        0.0
    };
    Savings {
        generic_kbps,
        per_title_kbps,
        saving_pct,
        compared_rungs: matched.len(),
    }
}
