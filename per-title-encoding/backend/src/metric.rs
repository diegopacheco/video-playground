use crate::error::{AppError, Result};
use crate::ffmpeg;
use crate::model::Resolution;
use std::path::Path;

pub trait QualityMetric: Send + Sync {
    fn name(&self) -> &'static str;
    fn min_gain(&self) -> f64;
    fn measure(&self, reference: &Path, distorted: &Path, display: Resolution) -> Result<f64>;
}

fn compare(reference: &Path, distorted: &Path, display: Resolution, filter: &str) -> Result<String> {
    let graph = format!(
        "[0:v]scale={}:{}:flags=bicubic[d];[d][1:v]{filter}",
        display.width, display.height
    );
    ffmpeg::run(&ffmpeg::args(&[
        "-i",
        &distorted.to_string_lossy(),
        "-i",
        &reference.to_string_lossy(),
        "-filter_complex",
        &graph,
        "-f",
        "null",
        "-",
    ]))
}

fn number_after(output: &str, marker: &str) -> Option<f64> {
    output.lines().rev().find_map(|line| {
        let start = line.find(marker)? + marker.len();
        let rest = line[start..].trim_start();
        let end = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(rest.len());
        rest[..end].parse().ok()
    })
}

pub fn parse_vmaf(output: &str) -> Result<f64> {
    number_after(output, "VMAF score:").ok_or(AppError::Parse {
        metric: "vmaf".into(),
    })
}

pub fn parse_ssim(output: &str) -> Result<f64> {
    output
        .lines()
        .rev()
        .find(|l| l.contains("SSIM Y:"))
        .and_then(|l| number_after(l, "All:"))
        .ok_or(AppError::Parse {
            metric: "ssim".into(),
        })
}

pub struct Vmaf {
    pub threads: u32,
}

impl QualityMetric for Vmaf {
    fn name(&self) -> &'static str {
        "vmaf"
    }

    fn min_gain(&self) -> f64 {
        0.5
    }

    fn measure(&self, reference: &Path, distorted: &Path, display: Resolution) -> Result<f64> {
        let filter = format!("libvmaf=n_threads={}", self.threads);
        parse_vmaf(&compare(reference, distorted, display, &filter)?)
    }
}

pub struct Ssim;

impl QualityMetric for Ssim {
    fn name(&self) -> &'static str {
        "ssim"
    }

    fn min_gain(&self) -> f64 {
        0.0005
    }

    fn measure(&self, reference: &Path, distorted: &Path, display: Resolution) -> Result<f64> {
        parse_ssim(&compare(reference, distorted, display, "ssim")?)
    }
}
