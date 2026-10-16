use std::path::Path;

use crate::ads::{AdSegment, Catalog, Creative};
use crate::error::{Result, SsaiError};
use crate::playlist::Playlist;

struct Spec {
    id: &'static str,
    advertiser: &'static str,
    regions: &'static [&'static str],
}

const SPECS: [Spec; 4] = [
    Spec { id: "acme", advertiser: "Acme Rockets", regions: &[] },
    Spec { id: "globex", advertiser: "Globex Cola", regions: &["us", "eu"] },
    Spec { id: "initech", advertiser: "Initech Bank", regions: &["br"] },
    Spec { id: "umbrella", advertiser: "Umbrella Travel", regions: &["eu", "br"] },
];

pub const SLATE_ID: &str = "slate";

pub fn load_catalog(ads_dir: &Path) -> Result<Catalog> {
    let creatives = SPECS
        .iter()
        .map(|s| {
            let regions = s.regions.iter().map(|r| r.to_string()).collect();
            load_creative(ads_dir, s.id, s.advertiser, regions, true)
        })
        .collect::<Result<Vec<_>>>()?;
    let slate = load_creative(ads_dir, SLATE_ID, "House Slate", Vec::new(), false)?;
    Ok(Catalog { creatives, slate })
}

fn load_creative(ads_dir: &Path, id: &str, advertiser: &str, regions: Vec<String>, tracked: bool) -> Result<Creative> {
    let path = ads_dir.join(id).join("index.m3u8");
    let text = std::fs::read_to_string(&path).map_err(|e| SsaiError::InvalidCatalog(format!("{}: {e}", path.display())))?;
    let playlist = Playlist::parse(&text)?;
    let segments: Vec<AdSegment> = playlist.segments().map(|(duration, uri)| AdSegment { duration, file: uri.into() }).collect();
    if segments.is_empty() {
        return Err(SsaiError::InvalidCatalog(format!("{id} has no segments")));
    }
    Ok(Creative { id: id.into(), advertiser: advertiser.into(), regions, segments, tracked })
}
