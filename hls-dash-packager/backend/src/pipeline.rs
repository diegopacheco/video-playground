use crate::alignment;
use crate::dash;
use crate::encoder::Encoder;
use crate::error::Result;
use crate::hls;
use crate::ladder::Ladder;
use crate::manifest::{Manifest, RenditionReport, SegmentFile};
use crate::packager::{INIT_SEGMENT, MEDIA_PLAYLIST, Packager};
use crate::probe::KeyframeProbe;
use std::fs;
use std::path::Path;

pub const MASTER_PLAYLIST: &str = "master.m3u8";
pub const DASH_MANIFEST: &str = "manifest.mpd";
pub const MANIFEST_JSON: &str = "manifest.json";

pub struct Pipeline {
    pub ladder: Ladder,
    pub encoder: Box<dyn Encoder>,
    pub packager: Box<dyn Packager>,
    pub probe: Box<dyn KeyframeProbe>,
}

impl Pipeline {
    pub fn run(&self, source: &Path, work_dir: &Path, out_dir: &Path) -> Result<Manifest> {
        self.ladder.validate()?;
        fs::create_dir_all(work_dir)?;
        fs::create_dir_all(out_dir)?;

        let mut reports = Vec::with_capacity(self.ladder.renditions.len());
        for rendition in &self.ladder.renditions {
            let encoded = work_dir.join(format!("{}.mp4", rendition.id));
            self.encoder.encode(source, rendition, &self.ladder, &encoded)?;
            let keyframes = self.probe.keyframes(&encoded)?;
            let rendition_dir = out_dir.join(&rendition.id);
            if rendition_dir.exists() {
                fs::remove_dir_all(&rendition_dir)?;
            }
            self.packager.package(&encoded, &self.ladder, &rendition_dir)?;
            reports.push(read_rendition(rendition, &rendition_dir, keyframes)?);
        }

        let tolerance = 0.5 / self.ladder.fps as f64;
        let alignment = alignment::check(&reports, tolerance);
        let duration = reports.first().map(|r| r.duration()).unwrap_or(0.0);

        fs::write(out_dir.join(MASTER_PLAYLIST), hls::master_playlist(&reports, self.ladder.fps))?;
        fs::write(out_dir.join(DASH_MANIFEST), dash::mpd(&self.ladder, &reports, duration))?;

        let manifest = Manifest {
            fps: self.ladder.fps,
            gop_frames: self.ladder.gop_frames,
            gop_seconds: self.ladder.gop_seconds(),
            segment_seconds: self.ladder.segment_seconds,
            duration,
            hls_master: MASTER_PLAYLIST.into(),
            dash_mpd: DASH_MANIFEST.into(),
            renditions: reports,
            alignment,
        };
        fs::write(out_dir.join(MANIFEST_JSON), serde_json::to_vec_pretty(&manifest)?)?;
        Ok(manifest)
    }
}

fn read_rendition(rendition: &crate::ladder::Rendition, dir: &Path, keyframes: Vec<f64>) -> Result<RenditionReport> {
    let playlist = fs::read_to_string(dir.join(MEDIA_PLAYLIST))?;
    let files = hls::parse_media_playlist(&playlist)?
        .into_iter()
        .map(|e| {
            let bytes = fs::metadata(dir.join(&e.uri))?.len();
            Ok(SegmentFile {
                uri: e.uri,
                duration: e.duration,
                bytes,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let init_bytes = fs::metadata(dir.join(INIT_SEGMENT))?.len();
    Ok(RenditionReport::build(rendition, init_bytes, files, keyframes))
}

pub fn load_manifest(out_dir: &Path) -> Result<Manifest> {
    let bytes = fs::read(out_dir.join(MANIFEST_JSON))?;
    Ok(serde_json::from_slice(&bytes)?)
}
