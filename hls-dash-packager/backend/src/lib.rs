pub mod alignment;
pub mod command;
pub mod dash;
pub mod encoder;
pub mod error;
pub mod hls;
pub mod ladder;
pub mod manifest;
pub mod mime;
pub mod packager;
pub mod pipeline;
pub mod probe;
pub mod server;

use encoder::FfmpegEncoder;
use ladder::Ladder;
use packager::FfmpegCmafPackager;
use pipeline::Pipeline;
use probe::FfprobeKeyframes;

pub fn ffmpeg_pipeline(ladder: Ladder) -> Pipeline {
    Pipeline {
        ladder,
        encoder: Box::new(FfmpegEncoder::default()),
        packager: Box::new(FfmpegCmafPackager),
        probe: Box::new(FfprobeKeyframes),
    }
}
