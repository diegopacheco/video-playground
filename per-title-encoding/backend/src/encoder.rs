use crate::error::Result;
use crate::ffmpeg;
use crate::model::Rendition;
use std::path::Path;

pub trait Encoder: Send + Sync {
    fn name(&self) -> String;
    fn encode(&self, source: &Path, rendition: &Rendition, output: &Path) -> Result<u64>;
}

pub struct X264Cbr {
    pub preset: String,
}

impl Default for X264Cbr {
    fn default() -> Self {
        Self {
            preset: "medium".into(),
        }
    }
}

impl X264Cbr {
    pub fn arguments(&self, source: &Path, rendition: &Rendition, output: &Path) -> Vec<String> {
        let rate = format!("{}k", rendition.kbps);
        let scale = format!(
            "scale={}:{}:flags=bicubic",
            rendition.resolution.width, rendition.resolution.height
        );
        ffmpeg::args(&[
            "-y",
            "-loglevel",
            "error",
            "-i",
            &source.to_string_lossy(),
            "-vf",
            &scale,
            "-c:v",
            "libx264",
            "-preset",
            &self.preset,
            "-b:v",
            &rate,
            "-minrate",
            &rate,
            "-maxrate",
            &rate,
            "-bufsize",
            &rate,
            "-x264-params",
            "nal-hrd=cbr",
            "-an",
            &output.to_string_lossy(),
        ])
    }
}

impl Encoder for X264Cbr {
    fn name(&self) -> String {
        format!("libx264 CBR preset={}", self.preset)
    }

    fn encode(&self, source: &Path, rendition: &Rendition, output: &Path) -> Result<u64> {
        ffmpeg::run(&self.arguments(source, rendition, output))?;
        Ok(std::fs::metadata(output)?.len())
    }
}
