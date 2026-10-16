pub mod ads;
pub mod catalog;
pub mod error;
pub mod http;
pub mod playlist;
pub mod stitcher;
pub mod tracking;
pub mod users;

use std::path::Path;
use std::sync::Arc;

use ads::TargetedAdDecision;
use error::{Result, SsaiError};
use http::AppState;
use playlist::Playlist;
use stitcher::{ManifestStitcher, UriScheme};
use tracking::MemoryBeaconSink;

pub fn build_state(media_dir: &Path) -> Result<AppState> {
    let content_path = media_dir.join("content").join("content.m3u8");
    let raw = std::fs::read_to_string(&content_path).map_err(|e| SsaiError::NotFound(format!("{}: {e}", content_path.display())))?;
    let content = Playlist::parse(&raw)?;
    let catalog = Arc::new(catalog::load_catalog(&media_dir.join("ads"))?);
    let uris = UriScheme {
        content_base: "/media/content".into(),
        media_base: "/media/ads".into(),
        tracking_base: "/api/track".into(),
    };
    let stitcher = ManifestStitcher::new(Arc::new(TargetedAdDecision::new(catalog.clone())), uris);
    Ok(AppState { raw, content, stitcher, catalog, beacons: Arc::new(MemoryBeaconSink::default()), media_dir: media_dir.to_path_buf() })
}
