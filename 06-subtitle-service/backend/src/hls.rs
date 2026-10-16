use crate::error::SubError;

#[derive(Debug, Clone, PartialEq)]
pub struct Rendition {
    pub locale: String,
    pub name: String,
    pub default: bool,
}

pub fn master_playlist(
    video_master: &str,
    video_base: &str,
    renditions: &[Rendition],
) -> Result<String, SubError> {
    let mut lines = video_master.lines().map(str::trim);
    let mut stream_inf = None;
    let mut uri = None;
    while let Some(line) = lines.next() {
        if line.starts_with("#EXT-X-STREAM-INF:") {
            stream_inf = Some(line.to_string());
            uri = lines.find(|l| !l.is_empty() && !l.starts_with('#'));
            break;
        }
    }
    let (Some(stream_inf), Some(uri)) = (stream_inf, uri) else {
        return Err(SubError::MediaMissing(
            "video master playlist has no EXT-X-STREAM-INF".into(),
        ));
    };
    let mut out = String::from("#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-INDEPENDENT-SEGMENTS\n");
    for r in renditions {
        out.push_str(&format!(
            "#EXT-X-MEDIA:TYPE=SUBTITLES,GROUP-ID=\"subs\",NAME=\"{}\",LANGUAGE=\"{}\",DEFAULT={},AUTOSELECT=YES,FORCED=NO,URI=\"subs/{}/index.m3u8\"\n",
            r.name.replace('"', "'"),
            r.locale,
            if r.default { "YES" } else { "NO" },
            r.locale
        ));
    }
    out.push_str(&format!("{stream_inf},SUBTITLES=\"subs\"\n{video_base}{uri}\n"));
    Ok(out)
}

pub fn playlist_duration(media_playlist: &str) -> Result<f64, SubError> {
    let mut total = 0.0;
    for line in media_playlist.lines() {
        if let Some(rest) = line.trim().strip_prefix("#EXTINF:") {
            let value = rest.split(',').next().unwrap_or("");
            total += value
                .trim()
                .parse::<f64>()
                .map_err(|_| SubError::MediaMissing(format!("bad EXTINF '{line}'")))?;
        }
    }
    if total <= 0.0 {
        return Err(SubError::MediaMissing("video playlist has no segments".into()));
    }
    Ok(total)
}

pub fn subtitle_playlist(duration: f64) -> String {
    format!(
        "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:{}\n#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-PLAYLIST-TYPE:VOD\n#EXTINF:{duration:.3},\nsubs.vtt\n#EXT-X-ENDLIST\n",
        duration.ceil() as u64
    )
}

pub fn with_timestamp_map(vtt: &str) -> String {
    let body = vtt.strip_prefix("WEBVTT").unwrap_or(vtt);
    format!("WEBVTT\nX-TIMESTAMP-MAP=MPEGTS:0,LOCAL:00:00:00.000{body}")
}
