#![allow(dead_code)]

use per_title::config::{AnalysisConfig, Title};
use per_title::encoder::Encoder;
use per_title::error::{AppError, Result};
use per_title::metric::QualityMetric;
use per_title::model::{Rendition, Resolution};
use per_title::pipeline::Pipeline;
use per_title::source::SourceGenerator;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct TouchSource;

impl SourceGenerator for TouchSource {
    fn generate(&self, _title: &Title, output: &Path) -> Result<()> {
        std::fs::write(output, b"source")?;
        Ok(())
    }
}

pub struct RecordingEncoder {
    pub fail: bool,
}

impl Encoder for RecordingEncoder {
    fn name(&self) -> String {
        "recording".into()
    }

    fn encode(&self, _source: &Path, r: &Rendition, output: &Path) -> Result<u64> {
        if self.fail {
            return Err(AppError::Ffmpeg("encoder crashed".into()));
        }
        std::fs::write(output, format!("{} {}", r.resolution.height, r.kbps))?;
        Ok(r.kbps as u64 * 1000 / 8 * 2)
    }
}

pub struct ModelMetric;

impl QualityMetric for ModelMetric {
    fn name(&self) -> &'static str {
        "vmaf"
    }

    fn min_gain(&self) -> f64 {
        0.5
    }

    fn measure(&self, _reference: &Path, distorted: &Path, _display: Resolution) -> Result<f64> {
        let text = std::fs::read_to_string(distorted)?;
        let mut parts = text.split(' ').filter_map(|s| s.parse::<f64>().ok());
        let height = parts.next().unwrap_or(1.0);
        let kbps = parts.next().unwrap_or(1.0);
        let ceiling = height / 360.0 * 100.0;
        Ok(ceiling * (1.0 - (-kbps / (height * 2.0)).exp()))
    }
}

pub fn small_config() -> AnalysisConfig {
    AnalysisConfig {
        duration_secs: 2.0,
        resolutions: vec![Resolution::new(320, 180), Resolution::new(640, 360)],
        bitrates_kbps: vec![100, 400, 1600],
        generic_ladder: vec![
            Rendition {
                resolution: Resolution::new(320, 180),
                kbps: 400,
            },
            Rendition {
                resolution: Resolution::new(640, 360),
                kbps: 1600,
            },
        ],
        workers: 2,
        ..AnalysisConfig::default()
    }
}

pub fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("per-title-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

pub fn fake_pipeline(name: &str, fail: bool) -> Pipeline {
    Pipeline {
        config: small_config(),
        source: Arc::new(TouchSource),
        encoder: Arc::new(RecordingEncoder { fail }),
        metrics: vec![Arc::new(ModelMetric)],
        work_dir: temp_dir(name),
    }
}
