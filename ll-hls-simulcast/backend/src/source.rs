use crate::clock::{STRIP_HEIGHT, STRIP_WIDTH, now_ms, render_strip, time_of_day};
use crate::error::{Error, Result};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWriteExt};
use tokio::process::{ChildStdin, Command};
use tokio::task::JoinHandle;

pub struct RunningSource {
    pub epoch_ms: i64,
    pub output: Box<dyn AsyncRead + Send + Unpin>,
    pub task: JoinHandle<Result<()>>,
}

pub trait LiveSource {
    fn start(&self) -> Result<RunningSource>;
}

pub struct FfmpegSource {
    pub fps: u32,
    pub segment_seconds: u32,
    pub part_ms: u32,
}

impl FfmpegSource {
    pub fn args(&self) -> Vec<String> {
        let gop = (self.fps * self.segment_seconds).to_string();
        let filter = "[0:v][1:v]overlay=x=(W-w)/2:y=H-h-40:shortest=1[v]".to_string();
        [
            "-hide_banner", "-loglevel", "error",
            "-f", "lavfi", "-i", &format!("testsrc2=size=1280x720:rate={}", self.fps),
            "-f", "rawvideo", "-pix_fmt", "gray",
            "-video_size", &format!("{STRIP_WIDTH}x{STRIP_HEIGHT}"),
            "-framerate", &self.fps.to_string(), "-i", "pipe:0",
            "-filter_complex", &filter, "-map", "[v]",
            "-c:v", "libx264", "-preset", "ultrafast", "-tune", "zerolatency",
            "-profile:v", "baseline", "-pix_fmt", "yuv420p",
            "-g", &gop, "-keyint_min", &gop, "-sc_threshold", "0",
            "-f", "mp4", "-movflags", "empty_moov+default_base_moof+frag_keyframe",
            "-frag_duration", &(self.part_ms * 1000).to_string(),
            "-flush_packets", "1", "pipe:1",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }
}

async fn feed_clock(mut stdin: ChildStdin, fps: u32) -> Result<()> {
    let mut ticker = tokio::time::interval(Duration::from_secs_f64(1.0 / fps as f64));
    loop {
        ticker.tick().await;
        stdin.write_all(&render_strip(&time_of_day(now_ms()))).await?;
    }
}

impl LiveSource for FfmpegSource {
    fn start(&self) -> Result<RunningSource> {
        let mut child = Command::new("ffmpeg")
            .args(self.args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| Error::Source(format!("cannot spawn ffmpeg: {e}")))?;
        let stdin = child.stdin.take().ok_or_else(|| Error::Source("ffmpeg stdin missing".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| Error::Source("ffmpeg stdout missing".into()))?;
        let fps = self.fps;
        let epoch_ms = now_ms();
        let task = tokio::spawn(async move {
            let fed = feed_clock(stdin, fps).await;
            let _ = child.kill().await;
            fed
        });
        Ok(RunningSource { epoch_ms, output: Box::new(stdout), task })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> FfmpegSource {
        FfmpegSource { fps: 30, segment_seconds: 2, part_ms: 200 }
    }

    fn value_after(args: &[String], flag: &str) -> String {
        let at = args.iter().position(|a| a == flag).unwrap();
        args[at + 1].clone()
    }

    #[test]
    fn gop_equals_segment_length_so_every_segment_starts_on_a_keyframe() {
        let args = source().args();
        assert_eq!(value_after(&args, "-g"), "60");
        assert_eq!(value_after(&args, "-keyint_min"), "60");
        assert_eq!(value_after(&args, "-sc_threshold"), "0");
    }

    #[test]
    fn fragment_duration_matches_the_part_target() {
        assert_eq!(value_after(&source().args(), "-frag_duration"), "200000");
    }

    #[test]
    fn clock_strip_geometry_matches_the_rawvideo_input_declaration() {
        assert_eq!(value_after(&source().args(), "-video_size"), "960x120");
    }

    #[test]
    fn encoder_avoids_lookahead_and_b_frames_that_would_add_latency() {
        let args = source().args();
        assert_eq!(value_after(&args, "-tune"), "zerolatency");
        assert_eq!(value_after(&args, "-profile:v"), "baseline");
    }
}
