use std::path::PathBuf;
use std::time::Instant;

use serde::Serialize;

use crate::ass::SubtitleParser;
use crate::error::SubError;
use crate::webvtt::{LossReport, convert};

pub trait TrackSource: Send + Sync {
    fn locales(&self) -> Result<Vec<String>, SubError>;
    fn load(&self, locale: &str) -> Result<String, SubError>;
}

pub struct FsTrackSource {
    dir: PathBuf,
}

impl FsTrackSource {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        FsTrackSource { dir: dir.into() }
    }
}

impl TrackSource for FsTrackSource {
    fn locales(&self) -> Result<Vec<String>, SubError> {
        let entries = std::fs::read_dir(&self.dir)
            .map_err(|e| SubError::Io(format!("{}: {e}", self.dir.display())))?;
        let mut locales = Vec::new();
        for entry in entries {
            let path = entry.map_err(|e| SubError::Io(e.to_string()))?.path();
            let is_sub = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("ass") || e.eq_ignore_ascii_case("ssa"));
            if let (true, Some(stem)) = (is_sub, path.file_stem().and_then(|s| s.to_str())) {
                locales.push(stem.to_string());
            }
        }
        locales.sort();
        Ok(locales)
    }

    fn load(&self, locale: &str) -> Result<String, SubError> {
        for ext in ["ass", "ssa"] {
            let path = self.dir.join(format!("{locale}.{ext}"));
            if path.exists() {
                return std::fs::read_to_string(&path)
                    .map_err(|e| SubError::Io(format!("{}: {e}", path.display())));
            }
        }
        Err(SubError::UnknownLocale(locale.to_string()))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub locale: String,
    pub name: String,
    pub ass_bytes: usize,
    pub vtt_bytes: usize,
    pub convert_micros: u128,
    pub report: LossReport,
    #[serde(skip)]
    pub ass: String,
    #[serde(skip)]
    pub vtt: String,
}

pub struct Catalog {
    tracks: Vec<Track>,
}

pub const DEFAULT_LOCALE: &str = "en-US";

impl Catalog {
    pub fn build(source: &dyn TrackSource, parser: &dyn SubtitleParser) -> Result<Self, SubError> {
        let mut tracks = Vec::new();
        for locale in source.locales()? {
            let ass = source.load(&locale)?;
            let started = Instant::now();
            let script = parser.parse(&ass).map_err(|e| match e {
                SubError::Parse { line, msg } => SubError::Parse {
                    line,
                    msg: format!("{locale}: {msg}"),
                },
                other => other,
            })?;
            let conversion = convert(&script);
            let convert_micros = started.elapsed().as_micros();
            let name = if script.info.title.is_empty() {
                locale.clone()
            } else {
                script.info.title.clone()
            };
            tracks.push(Track {
                locale,
                name,
                ass_bytes: ass.len(),
                vtt_bytes: conversion.vtt.len(),
                convert_micros,
                report: conversion.report,
                ass,
                vtt: conversion.vtt,
            });
        }
        Ok(Catalog { tracks })
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn get(&self, locale: &str) -> Result<&Track, SubError> {
        self.tracks
            .iter()
            .find(|t| t.locale == locale)
            .ok_or_else(|| SubError::UnknownLocale(locale.to_string()))
    }
}
