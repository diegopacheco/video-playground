#![allow(dead_code)]

use std::sync::Arc;

use ssai::ads::{AdSegment, Catalog, Creative, TargetedAdDecision};
use ssai::stitcher::{ManifestStitcher, UriScheme};

pub const CONTENT: &str = "#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:2
#EXT-X-MEDIA-SEQUENCE:0
#EXT-X-PLAYLIST-TYPE:VOD
#EXTINF:2.000000,
seg_000.ts
#EXTINF:2.000000,
seg_001.ts
#EXT-X-CUE-OUT:DURATION=12
#EXTINF:2.000000,
seg_002.ts
#EXTINF:2.000000,
seg_003.ts
#EXTINF:2.000000,
seg_004.ts
#EXTINF:2.000000,
seg_005.ts
#EXTINF:2.000000,
seg_006.ts
#EXTINF:2.000000,
seg_007.ts
#EXT-X-CUE-IN
#EXTINF:2.000000,
seg_008.ts
#EXTINF:2.000000,
seg_009.ts
#EXT-X-DATERANGE:ID=\"ad-break-2\",START-DATE=\"2026-01-01T00:00:20Z\",PLANNED-DURATION=8,SCTE35-OUT=0xFC302000
#EXTINF:2.000000,
seg_010.ts
#EXTINF:2.000000,
seg_011.ts
#EXTINF:2.000000,
seg_012.ts
#EXTINF:2.000000,
seg_013.ts
#EXTINF:2.000000,
seg_014.ts
#EXT-X-ENDLIST
";

pub fn creative(id: &str, regions: &[&str], seconds: usize, tracked: bool) -> Creative {
    Creative {
        id: id.into(),
        advertiser: format!("{id} inc"),
        regions: regions.iter().map(|r| r.to_string()).collect(),
        segments: (0..seconds / 2).map(|i| AdSegment { duration: 2.0, file: format!("seg_{i:03}.ts") }).collect(),
        tracked,
    }
}

pub fn catalog() -> Catalog {
    Catalog {
        creatives: vec![
            creative("acme", &[], 6, true),
            creative("globex", &["us"], 4, true),
            creative("initech", &["br"], 8, true),
        ],
        slate: creative("slate", &[], 2, false),
    }
}

pub fn stitcher() -> ManifestStitcher {
    let uris = UriScheme {
        content_base: "/media/content".into(),
        media_base: "/media/ads".into(),
        tracking_base: "/api/track".into(),
    };
    ManifestStitcher::new(Arc::new(TargetedAdDecision::new(Arc::new(catalog()))), uris)
}
