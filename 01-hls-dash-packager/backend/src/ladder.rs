use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rendition {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub bitrate_kbps: u32,
    pub level_idc: u8,
}

impl Rendition {
    pub fn new(id: &str, width: u32, height: u32, bitrate_kbps: u32, level_idc: u8) -> Self {
        Self {
            id: id.to_string(),
            width,
            height,
            bitrate_kbps,
            level_idc,
        }
    }

    pub fn codecs(&self) -> String {
        format!("avc1.6400{:02x}", self.level_idc)
    }

    pub fn level(&self) -> String {
        format!("{}.{}", self.level_idc / 10, self.level_idc % 10)
    }

    pub fn maxrate_kbps(&self) -> u32 {
        self.bitrate_kbps * 11 / 10
    }

    pub fn bufsize_kbps(&self) -> u32 {
        self.bitrate_kbps * 2
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ladder {
    pub fps: u32,
    pub gop_frames: u32,
    pub segment_seconds: u32,
    pub renditions: Vec<Rendition>,
}

impl Ladder {
    pub fn standard() -> Self {
        Self {
            fps: 24,
            gop_frames: 48,
            segment_seconds: 2,
            renditions: vec![
                Rendition::new("240p", 426, 240, 400, 21),
                Rendition::new("480p", 854, 480, 1000, 30),
                Rendition::new("720p", 1280, 720, 2800, 31),
                Rendition::new("1080p", 1920, 1080, 5000, 40),
            ],
        }
    }

    pub fn gop_seconds(&self) -> f64 {
        self.gop_frames as f64 / self.fps as f64
    }

    pub fn validate(&self) -> Result<()> {
        if self.renditions.is_empty() {
            return Err(AppError::Ladder("ladder has no renditions".into()));
        }
        if self.gop_frames == 0 || self.fps == 0 || self.segment_seconds == 0 {
            return Err(AppError::Ladder("fps, gop and segment length must be positive".into()));
        }
        if !(self.fps * self.segment_seconds).is_multiple_of(self.gop_frames) {
            return Err(AppError::Ladder(format!(
                "segment of {}s at {}fps is {} frames, not a multiple of the {}-frame GOP, so segments could not start on a keyframe",
                self.segment_seconds,
                self.fps,
                self.fps * self.segment_seconds,
                self.gop_frames
            )));
        }
        for pair in self.renditions.windows(2) {
            if pair[1].height <= pair[0].height || pair[1].bitrate_kbps <= pair[0].bitrate_kbps {
                return Err(AppError::Ladder(format!(
                    "{} must have more pixels and more bitrate than {}",
                    pair[1].id, pair[0].id
                )));
            }
        }
        for r in &self.renditions {
            if r.width % 2 != 0 || r.height % 2 != 0 {
                return Err(AppError::Ladder(format!("{} must have even dimensions for yuv420p", r.id)));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_ladder_is_valid_so_packaging_can_start() {
        assert!(Ladder::standard().validate().is_ok());
    }

    #[test]
    fn segment_not_multiple_of_gop_is_rejected_because_segments_would_start_mid_gop() {
        let mut ladder = Ladder::standard();
        ladder.gop_frames = 36;
        let err = ladder.validate().unwrap_err().to_string();
        assert!(err.contains("not a multiple"), "{err}");
    }

    #[test]
    fn segment_may_hold_several_whole_gops() {
        let mut ladder = Ladder::standard();
        ladder.gop_frames = 24;
        assert!(ladder.validate().is_ok());
    }

    #[test]
    fn ladder_must_climb_in_bitrate_so_abr_can_switch_up() {
        let mut ladder = Ladder::standard();
        ladder.renditions[2].bitrate_kbps = 500;
        assert!(ladder.validate().is_err());
    }

    #[test]
    fn odd_dimensions_are_rejected_for_420_chroma() {
        let mut ladder = Ladder::standard();
        ladder.renditions[0].width = 427;
        assert!(ladder.validate().is_err());
    }

    #[test]
    fn codecs_string_encodes_high_profile_and_level_for_player_capability_checks() {
        let ladder = Ladder::standard();
        let codecs: Vec<String> = ladder.renditions.iter().map(|r| r.codecs()).collect();
        assert_eq!(codecs, vec!["avc1.640015", "avc1.64001e", "avc1.64001f", "avc1.640028"]);
        assert_eq!(ladder.renditions[3].level(), "4.0");
    }

    #[test]
    fn vbv_caps_follow_target_bitrate() {
        let r = Rendition::new("x", 2, 2, 1000, 30);
        assert_eq!(r.maxrate_kbps(), 1100);
        assert_eq!(r.bufsize_kbps(), 2000);
    }
}
