use crate::manifest::RenditionReport;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlignmentReport {
    pub aligned: bool,
    pub boundaries_on_keyframes: bool,
    pub keyframe_times: Vec<f64>,
    pub issues: Vec<String>,
}

impl AlignmentReport {
    pub fn ok(&self) -> bool {
        self.aligned && self.boundaries_on_keyframes
    }
}

fn close(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() <= tolerance
}

pub fn check(renditions: &[RenditionReport], tolerance: f64) -> AlignmentReport {
    let mut issues = Vec::new();
    let reference = renditions.first().map(|r| r.keyframes.clone()).unwrap_or_default();
    let reference_id = renditions.first().map(|r| r.id.as_str()).unwrap_or("");

    let mut aligned = true;
    for r in renditions.iter().skip(1) {
        if r.keyframes.len() != reference.len() {
            aligned = false;
            issues.push(format!(
                "{} has {} keyframes but {} has {}",
                r.id,
                r.keyframes.len(),
                reference_id,
                reference.len()
            ));
            continue;
        }
        if let Some((i, (a, b))) = r
            .keyframes
            .iter()
            .zip(&reference)
            .enumerate()
            .find(|(_, (a, b))| !close(**a, **b, tolerance))
        {
            aligned = false;
            issues.push(format!("{} keyframe #{i} at {a:.3}s but {reference_id} at {b:.3}s", r.id));
        }
    }

    let mut boundaries_on_keyframes = true;
    for r in renditions {
        for s in &r.segments {
            if !r.keyframes.iter().any(|k| close(*k, s.start, tolerance)) {
                boundaries_on_keyframes = false;
                issues.push(format!("{} segment {} starts at {:.3}s without a keyframe", r.id, s.index, s.start));
            }
        }
    }

    AlignmentReport {
        aligned,
        boundaries_on_keyframes,
        keyframe_times: reference,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ladder::Rendition;
    use crate::manifest::SegmentFile;

    fn report(id: &str, keyframes: Vec<f64>, durations: &[f64]) -> RenditionReport {
        let files = durations
            .iter()
            .map(|d| SegmentFile { uri: "s.m4s".into(), duration: *d, bytes: 1000 })
            .collect();
        RenditionReport::build(&Rendition::new(id, 2, 2, 1, 30), 0, files, keyframes)
    }

    #[test]
    fn identical_keyframes_let_the_player_switch_at_any_segment_boundary() {
        let r = check(
            &[report("240p", vec![0.0, 2.0, 4.0], &[2.0, 2.0, 2.0]), report("480p", vec![0.0, 2.0, 4.0], &[2.0, 2.0, 2.0])],
            0.02,
        );
        assert!(r.ok(), "{:?}", r.issues);
        assert_eq!(r.keyframe_times, vec![0.0, 2.0, 4.0]);
    }

    #[test]
    fn shifted_keyframe_is_reported_because_a_switch_there_would_glitch() {
        let r = check(
            &[report("240p", vec![0.0, 2.0, 4.0], &[2.0, 2.0]), report("720p", vec![0.0, 2.5, 4.0], &[2.0, 2.0])],
            0.02,
        );
        assert!(!r.aligned);
        assert!(r.issues.iter().any(|i| i.contains("720p keyframe #1")), "{:?}", r.issues);
    }

    #[test]
    fn extra_scene_cut_keyframe_breaks_alignment() {
        let r = check(&[report("a", vec![0.0, 2.0], &[2.0]), report("b", vec![0.0, 1.3, 2.0], &[2.0])], 0.02);
        assert!(!r.aligned);
    }

    #[test]
    fn segment_starting_between_keyframes_is_not_independently_decodable() {
        let r = check(&[report("a", vec![0.0, 4.0], &[2.0, 2.0])], 0.02);
        assert!(r.aligned);
        assert!(!r.boundaries_on_keyframes);
        assert!(r.issues[0].contains("segment 2 starts at 2.000s"));
    }

    #[test]
    fn sub_frame_jitter_is_tolerated() {
        let r = check(&[report("a", vec![0.0, 2.0], &[2.0, 2.0]), report("b", vec![0.0, 2.004], &[2.0, 2.0])], 0.02);
        assert!(r.ok(), "{:?}", r.issues);
    }
}
