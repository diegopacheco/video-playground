use crate::command;
use crate::error::Result;
use crate::ladder::Ladder;
use std::path::Path;

pub const INIT_SEGMENT: &str = "init.mp4";
pub const MEDIA_PLAYLIST: &str = "playlist.m3u8";
pub const SEGMENT_PATTERN: &str = "seg_%05d.m4s";

pub trait Packager: Send + Sync {
    fn package(&self, input: &Path, ladder: &Ladder, out_dir: &Path) -> Result<()>;
}

pub struct FfmpegCmafPackager;

impl FfmpegCmafPackager {
    pub fn args(&self, input: &Path, ladder: &Ladder, out_dir: &Path) -> Vec<String> {
        vec![
            "-y".into(),
            "-v".into(),
            "error".into(),
            "-i".into(),
            input.display().to_string(),
            "-c".into(),
            "copy".into(),
            "-f".into(),
            "hls".into(),
            "-hls_time".into(),
            ladder.segment_seconds.to_string(),
            "-hls_playlist_type".into(),
            "vod".into(),
            "-hls_segment_type".into(),
            "fmp4".into(),
            "-hls_flags".into(),
            "independent_segments".into(),
            "-hls_fmp4_init_filename".into(),
            INIT_SEGMENT.into(),
            "-start_number".into(),
            "1".into(),
            "-hls_segment_filename".into(),
            out_dir.join(SEGMENT_PATTERN).display().to_string(),
            out_dir.join(MEDIA_PLAYLIST).display().to_string(),
        ]
    }
}

impl Packager for FfmpegCmafPackager {
    fn package(&self, input: &Path, ladder: &Ladder, out_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(out_dir)?;
        command::run("ffmpeg", &self.args(input, ladder, out_dir)).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_are_fmp4_without_reencoding_so_hls_and_dash_share_the_same_bytes() {
        let args = FfmpegCmafPackager.args(Path::new("in.mp4"), &Ladder::standard(), Path::new("out"));
        let joined = args.join(" ");
        assert!(joined.contains("-c copy"));
        assert!(joined.contains("-hls_segment_type fmp4"));
        assert!(joined.contains("-hls_time 2"));
        assert!(joined.contains("-start_number 1"));
    }
}
