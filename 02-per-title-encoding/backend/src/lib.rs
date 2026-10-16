pub mod api;
pub mod config;
pub mod encoder;
pub mod error;
pub mod ffmpeg;
pub mod hull;
pub mod ladder;
pub mod metric;
pub mod model;
pub mod pipeline;
pub mod source;
pub mod store;

use config::AnalysisConfig;
use encoder::X264Cbr;
use metric::{Ssim, Vmaf};
use pipeline::Pipeline;
use source::LavfiSource;
use std::path::PathBuf;
use std::sync::Arc;

pub fn ffmpeg_pipeline(work_dir: PathBuf) -> Pipeline {
    Pipeline {
        config: AnalysisConfig::default(),
        source: Arc::new(LavfiSource),
        encoder: Arc::new(X264Cbr::default()),
        metrics: vec![Arc::new(Vmaf { threads: 4 }), Arc::new(Ssim)],
        work_dir,
    }
}
