use crate::command;
use crate::error::Result;
use crate::ladder::{Ladder, Rendition};
use std::path::Path;

pub trait Encoder: Send + Sync {
    fn encode(&self, source: &Path, rendition: &Rendition, ladder: &Ladder, output: &Path) -> Result<()>;
}

pub struct FfmpegEncoder {
    pub preset: String,
}

impl Default for FfmpegEncoder {
    fn default() -> Self {
        Self {
            preset: "veryfast".into(),
        }
    }
}

impl FfmpegEncoder {
    pub fn args(&self, source: &Path, rendition: &Rendition, ladder: &Ladder, output: &Path) -> Vec<String> {
        let gop = ladder.gop_frames.to_string();
        vec![
            "-y".into(),
            "-v".into(),
            "error".into(),
            "-i".into(),
            source.display().to_string(),
            "-an".into(),
            "-vf".into(),
            format!("scale={}:{}:flags=lanczos,fps={}", rendition.width, rendition.height, ladder.fps),
            "-c:v".into(),
            "libx264".into(),
            "-preset".into(),
            self.preset.clone(),
            "-profile:v".into(),
            "high".into(),
            "-level:v".into(),
            rendition.level(),
            "-pix_fmt".into(),
            "yuv420p".into(),
            "-b:v".into(),
            format!("{}k", rendition.bitrate_kbps),
            "-maxrate".into(),
            format!("{}k", rendition.maxrate_kbps()),
            "-bufsize".into(),
            format!("{}k", rendition.bufsize_kbps()),
            "-g".into(),
            gop.clone(),
            "-keyint_min".into(),
            gop,
            "-sc_threshold".into(),
            "0".into(),
            "-movflags".into(),
            "+faststart".into(),
            output.display().to_string(),
        ]
    }
}

impl Encoder for FfmpegEncoder {
    fn encode(&self, source: &Path, rendition: &Rendition, ladder: &Ladder, output: &Path) -> Result<()> {
        command::run("ffmpeg", &self.args(source, rendition, ladder, output)).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value_after(args: &[String], flag: &str) -> String {
        let i = args.iter().position(|a| a == flag).expect(flag);
        args[i + 1].clone()
    }

    #[test]
    fn gop_is_fixed_and_scene_cut_disabled_so_every_rendition_cuts_keyframes_at_the_same_frames() {
        let ladder = Ladder::standard();
        let args = FfmpegEncoder::default().args(Path::new("in.mp4"), &ladder.renditions[0], &ladder, Path::new("out.mp4"));
        assert_eq!(value_after(&args, "-g"), "48");
        assert_eq!(value_after(&args, "-keyint_min"), "48");
        assert_eq!(value_after(&args, "-sc_threshold"), "0");
    }

    #[test]
    fn frame_rate_is_forced_so_gop_frames_map_to_the_same_wall_clock_time() {
        let ladder = Ladder::standard();
        let args = FfmpegEncoder::default().args(Path::new("in.mp4"), &ladder.renditions[1], &ladder, Path::new("out.mp4"));
        assert_eq!(value_after(&args, "-vf"), "scale=854:480:flags=lanczos,fps=24");
    }

    #[test]
    fn rate_control_is_capped_so_segment_sizes_stay_predictable() {
        let ladder = Ladder::standard();
        let args = FfmpegEncoder::default().args(Path::new("in.mp4"), &ladder.renditions[3], &ladder, Path::new("out.mp4"));
        assert_eq!(value_after(&args, "-b:v"), "5000k");
        assert_eq!(value_after(&args, "-maxrate"), "5500k");
        assert_eq!(value_after(&args, "-bufsize"), "10000k");
        assert_eq!(value_after(&args, "-level:v"), "4.0");
    }
}
