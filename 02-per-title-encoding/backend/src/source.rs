use crate::config::Title;
use crate::error::Result;
use crate::ffmpeg;
use std::path::Path;

pub trait SourceGenerator: Send + Sync {
    fn generate(&self, title: &Title, output: &Path) -> Result<()>;
}

pub struct LavfiSource;

impl SourceGenerator for LavfiSource {
    fn generate(&self, title: &Title, output: &Path) -> Result<()> {
        if output.exists() {
            return Ok(());
        }
        let out = output.to_string_lossy();
        let mut a = ffmpeg::args(&["-y", "-loglevel", "error", "-filter_complex"]);
        a.push(title.graph.clone());
        a.extend(ffmpeg::args(&[
            "-c:v", "libx264", "-preset", "veryfast", "-qp", "0", "-pix_fmt", "yuv420p", &out,
        ]));
        ffmpeg::run(&a).map(|_| ())
    }
}
