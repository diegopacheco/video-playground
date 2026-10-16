use crate::alignment::AlignmentReport;
use crate::ladder::Rendition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SegmentInfo {
    pub index: u32,
    pub uri: String,
    pub start: f64,
    pub duration: f64,
    pub bytes: u64,
    pub kbps: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenditionReport {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub target_kbps: u32,
    pub codecs: String,
    pub playlist: String,
    pub init_bytes: u64,
    pub total_bytes: u64,
    pub average_kbps: f64,
    pub peak_kbps: f64,
    pub keyframes: Vec<f64>,
    pub segments: Vec<SegmentInfo>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub fps: u32,
    pub gop_frames: u32,
    pub gop_seconds: f64,
    pub segment_seconds: u32,
    pub duration: f64,
    pub hls_master: String,
    pub dash_mpd: String,
    pub renditions: Vec<RenditionReport>,
    pub alignment: AlignmentReport,
}

pub struct SegmentFile {
    pub uri: String,
    pub duration: f64,
    pub bytes: u64,
}

fn kbps(bytes: u64, seconds: f64) -> f64 {
    if seconds <= 0.0 {
        return 0.0;
    }
    (bytes as f64 * 8.0 / seconds / 1000.0 * 10.0).round() / 10.0
}

impl RenditionReport {
    pub fn build(rendition: &Rendition, init_bytes: u64, files: Vec<SegmentFile>, keyframes: Vec<f64>) -> Self {
        let mut start = 0.0;
        let segments: Vec<SegmentInfo> = files
            .into_iter()
            .enumerate()
            .map(|(i, f)| {
                let info = SegmentInfo {
                    index: i as u32 + 1,
                    uri: format!("{}/{}", rendition.id, f.uri),
                    start,
                    duration: f.duration,
                    bytes: f.bytes,
                    kbps: kbps(f.bytes, f.duration),
                };
                start += f.duration;
                info
            })
            .collect();
        let total_bytes: u64 = segments.iter().map(|s| s.bytes).sum();
        let duration: f64 = segments.iter().map(|s| s.duration).sum();
        let peak_kbps = segments.iter().map(|s| s.kbps).fold(0.0, f64::max);
        Self {
            id: rendition.id.clone(),
            width: rendition.width,
            height: rendition.height,
            target_kbps: rendition.bitrate_kbps,
            codecs: rendition.codecs(),
            playlist: format!("{}/playlist.m3u8", rendition.id),
            init_bytes,
            total_bytes,
            average_kbps: kbps(total_bytes, duration),
            peak_kbps,
            keyframes,
            segments,
        }
    }

    pub fn duration(&self) -> f64 {
        self.segments.iter().map(|s| s.duration).sum()
    }

    pub fn segment_starts(&self) -> Vec<f64> {
        self.segments.iter().map(|s| s.start).collect()
    }

    pub fn peak_bps(&self) -> u64 {
        (self.peak_kbps * 1000.0).round() as u64
    }

    pub fn average_bps(&self) -> u64 {
        (self.average_kbps * 1000.0).round() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files() -> Vec<SegmentFile> {
        vec![
            SegmentFile { uri: "seg_00001.m4s".into(), duration: 2.0, bytes: 250_000 },
            SegmentFile { uri: "seg_00002.m4s".into(), duration: 2.0, bytes: 500_000 },
            SegmentFile { uri: "seg_00003.m4s".into(), duration: 1.0, bytes: 125_000 },
        ]
    }

    #[test]
    fn segment_starts_accumulate_durations_so_they_can_be_matched_against_keyframes() {
        let r = RenditionReport::build(&Rendition::new("480p", 854, 480, 1000, 30), 800, files(), vec![]);
        assert_eq!(r.segment_starts(), vec![0.0, 2.0, 4.0]);
        assert_eq!(r.segments[1].uri, "480p/seg_00002.m4s");
    }

    #[test]
    fn peak_is_the_heaviest_segment_because_bandwidth_must_cover_the_worst_case() {
        let r = RenditionReport::build(&Rendition::new("480p", 854, 480, 1000, 30), 800, files(), vec![]);
        assert_eq!(r.segments[0].kbps, 1000.0);
        assert_eq!(r.peak_kbps, 2000.0);
        assert_eq!(r.average_kbps, 1400.0);
        assert_eq!(r.peak_bps(), 2_000_000);
        assert_eq!(r.total_bytes, 875_000);
    }
}
