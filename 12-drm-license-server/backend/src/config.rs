use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::DrmError;

#[derive(Debug, Deserialize)]
pub struct ContentEntry {
    pub id: String,
    pub kid: String,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct DrmConfig {
    pub session_secret: String,
    pub url_secret: String,
    pub contents: Vec<ContentEntry>,
}

pub struct Settings {
    pub port: u16,
    pub media_dir: PathBuf,
    pub drm: DrmConfig,
}

impl DrmConfig {
    pub fn load(path: &Path) -> Result<Self, DrmError> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| DrmError::Config(format!("{}: {e}", path.display())))?;
        serde_json::from_str(&text).map_err(|e| DrmError::Config(e.to_string()))
    }

    pub fn key_entries(&self) -> Vec<(String, String, String)> {
        self.contents
            .iter()
            .map(|c| (c.id.clone(), c.kid.clone(), c.key.clone()))
            .collect()
    }
}

impl Settings {
    pub fn from_env() -> Result<Self, DrmError> {
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8112".into())
            .parse()
            .map_err(|_| DrmError::Config("PORT must be a number".into()))?;
        let media_dir = PathBuf::from(std::env::var("MEDIA_DIR").unwrap_or_else(|_| "../media".into()));
        let drm = DrmConfig::load(&media_dir.join("drm.json"))?;
        Ok(Self { port, media_dir, drm })
    }
}
