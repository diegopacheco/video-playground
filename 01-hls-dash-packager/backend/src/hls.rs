use crate::error::{AppError, Result};
use crate::manifest::RenditionReport;

pub struct MediaEntry {
    pub uri: String,
    pub duration: f64,
}

pub fn parse_media_playlist(text: &str) -> Result<Vec<MediaEntry>> {
    let mut entries = Vec::new();
    let mut pending: Option<f64> = None;
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if let Some(rest) = line.strip_prefix("#EXTINF:") {
            let value = rest.split(',').next().unwrap_or("");
            pending = Some(
                value
                    .parse::<f64>()
                    .map_err(|e| AppError::Parse(format!("EXTINF `{value}`: {e}")))?,
            );
        } else if !line.starts_with('#') {
            let duration = pending
                .take()
                .ok_or_else(|| AppError::Parse(format!("segment `{line}` has no EXTINF")))?;
            entries.push(MediaEntry {
                uri: line.to_string(),
                duration,
            });
        }
    }
    if entries.is_empty() {
        return Err(AppError::Parse("media playlist has no segments".into()));
    }
    Ok(entries)
}

pub fn master_playlist(renditions: &[RenditionReport], fps: u32) -> String {
    let mut out = String::from("#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-INDEPENDENT-SEGMENTS\n");
    for r in renditions {
        out.push_str(&format!(
            "#EXT-X-STREAM-INF:BANDWIDTH={},AVERAGE-BANDWIDTH={},RESOLUTION={}x{},CODECS=\"{}\",FRAME-RATE={:.3}\n{}\n",
            r.peak_bps(),
            r.average_bps(),
            r.width,
            r.height,
            r.codecs,
            fps as f64,
            r.playlist
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ladder::Rendition;
    use crate::manifest::SegmentFile;

    const MEDIA: &str = "#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-TARGETDURATION:2\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:2.000000,\nseg_00001.m4s\n#EXTINF:1.500000,\nseg_00002.m4s\n#EXT-X-ENDLIST\n";

    #[test]
    fn media_playlist_yields_segment_uris_and_durations_in_order() {
        let entries = parse_media_playlist(MEDIA).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].uri, "seg_00002.m4s");
        assert_eq!(entries[1].duration, 1.5);
    }

    #[test]
    fn segment_without_duration_is_rejected() {
        assert!(parse_media_playlist("#EXTM3U\nseg.m4s\n").is_err());
    }

    #[test]
    fn master_advertises_peak_bandwidth_resolution_and_codecs_so_players_can_pick_a_level_before_downloading() {
        let files = vec![SegmentFile { uri: "seg_00001.m4s".into(), duration: 2.0, bytes: 250_000 }];
        let r = RenditionReport::build(&Rendition::new("480p", 854, 480, 1000, 30), 900, files, vec![0.0]);
        let master = master_playlist(&[r], 24);
        assert!(master.contains("#EXT-X-INDEPENDENT-SEGMENTS"));
        assert!(master.contains(
            "#EXT-X-STREAM-INF:BANDWIDTH=1000000,AVERAGE-BANDWIDTH=1000000,RESOLUTION=854x480,CODECS=\"avc1.64001e\",FRAME-RATE=24.000\n480p/playlist.m3u8\n"
        ));
    }
}
