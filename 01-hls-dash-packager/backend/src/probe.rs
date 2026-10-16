use crate::command;
use crate::error::{AppError, Result};
use std::path::Path;

pub trait KeyframeProbe: Send + Sync {
    fn keyframes(&self, file: &Path) -> Result<Vec<f64>>;
}

pub struct FfprobeKeyframes;

impl KeyframeProbe for FfprobeKeyframes {
    fn keyframes(&self, file: &Path) -> Result<Vec<f64>> {
        let args: Vec<String> = [
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-skip_frame",
            "nokey",
            "-show_entries",
            "frame=pts_time",
            "-of",
            "csv=p=0",
        ]
        .iter()
        .map(|s| s.to_string())
        .chain(std::iter::once(file.display().to_string()))
        .collect();
        parse_keyframe_csv(&command::run("ffprobe", &args)?)
    }
}

pub fn parse_keyframe_csv(text: &str) -> Result<Vec<f64>> {
    text.lines()
        .filter_map(|line| line.split(',').map(str::trim).find(|v| !v.is_empty()))
        .map(|v| v.parse::<f64>().map_err(|e| AppError::Parse(format!("keyframe time `{v}`: {e}"))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ffprobe_rows_including_side_data_trailing_commas() {
        let got = parse_keyframe_csv("0.000000,\n2.000000\n\n4.000000\n").unwrap();
        assert_eq!(got, vec![0.0, 2.0, 4.0]);
    }

    #[test]
    fn garbage_is_an_error_not_a_silent_skip() {
        assert!(parse_keyframe_csv("N/A\n").is_err());
    }
}
