use crate::error::{Result, SsaiError};

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Segment { duration: f64, uri: String },
    CueOut { id: String, duration: f64, raw: String },
    CueIn { raw: String },
    Discontinuity,
    Tag(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Playlist {
    pub header: Vec<String>,
    pub entries: Vec<Entry>,
    pub ended: bool,
}

const HEADER_TAGS: [&str; 5] = [
    "#EXT-X-VERSION",
    "#EXT-X-TARGETDURATION",
    "#EXT-X-MEDIA-SEQUENCE",
    "#EXT-X-PLAYLIST-TYPE",
    "#EXT-X-INDEPENDENT-SEGMENTS",
];

impl Playlist {
    pub fn parse(text: &str) -> Result<Playlist> {
        let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
        if lines.next() != Some("#EXTM3U") {
            return Err(SsaiError::InvalidPlaylist("missing #EXTM3U".into()));
        }
        let mut playlist = Playlist { header: vec!["#EXTM3U".into()], entries: Vec::new(), ended: false };
        let mut pending: Option<f64> = None;
        let mut cue_count = 0;
        for line in lines {
            if let Some(duration) = pending.take() {
                if line.starts_with('#') {
                    return Err(SsaiError::InvalidPlaylist(format!("EXTINF not followed by uri: {line}")));
                }
                playlist.entries.push(Entry::Segment { duration, uri: line.to_string() });
            } else if let Some(rest) = line.strip_prefix("#EXTINF:") {
                pending = Some(parse_number(rest.split(',').next().unwrap_or(""))?);
            } else if line == "#EXT-X-ENDLIST" {
                playlist.ended = true;
            } else if line == "#EXT-X-DISCONTINUITY" {
                playlist.entries.push(Entry::Discontinuity);
            } else if line.starts_with("#EXT-X-CUE-OUT") {
                cue_count += 1;
                let value = line.split_once(':').map(|(_, v)| v.trim_start_matches("DURATION=")).unwrap_or("0");
                playlist.entries.push(Entry::CueOut { id: format!("break-{cue_count}"), duration: parse_number(value)?, raw: line.into() });
            } else if line.starts_with("#EXT-X-CUE-IN") {
                playlist.entries.push(Entry::CueIn { raw: line.into() });
            } else if let Some(rest) = line.strip_prefix("#EXT-X-DATERANGE:") {
                playlist.entries.push(parse_daterange(rest, line)?);
            } else if playlist.entries.is_empty() && HEADER_TAGS.iter().any(|t| line.starts_with(t)) {
                playlist.header.push(line.into());
            } else {
                playlist.entries.push(Entry::Tag(line.into()));
            }
        }
        if pending.is_some() {
            return Err(SsaiError::InvalidPlaylist("EXTINF at end without uri".into()));
        }
        Ok(playlist)
    }

    pub fn segments(&self) -> impl Iterator<Item = (f64, &str)> {
        self.entries.iter().filter_map(|e| match e {
            Entry::Segment { duration, uri } => Some((*duration, uri.as_str())),
            _ => None,
        })
    }

    pub fn total_duration(&self) -> f64 {
        self.segments().map(|(d, _)| d).sum()
    }

    pub fn render(&self) -> String {
        let target = self.segments().map(|(d, _)| d.ceil() as u64).max().unwrap_or(0);
        let mut out = String::new();
        for line in &self.header {
            if line.starts_with("#EXT-X-TARGETDURATION") {
                out.push_str(&format!("#EXT-X-TARGETDURATION:{target}\n"));
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        for entry in &self.entries {
            match entry {
                Entry::Segment { duration, uri } => out.push_str(&format!("#EXTINF:{duration:.6},\n{uri}\n")),
                Entry::CueOut { raw, .. } | Entry::CueIn { raw } | Entry::Tag(raw) => {
                    out.push_str(raw);
                    out.push('\n');
                }
                Entry::Discontinuity => out.push_str("#EXT-X-DISCONTINUITY\n"),
            }
        }
        if self.ended {
            out.push_str("#EXT-X-ENDLIST\n");
        }
        out
    }
}

fn parse_number(value: &str) -> Result<f64> {
    value.trim().parse::<f64>().map_err(|_| SsaiError::InvalidPlaylist(format!("bad number: {value}")))
}

fn parse_daterange(attrs: &str, raw: &str) -> Result<Entry> {
    let pairs = split_attributes(attrs);
    let get = |key: &str| pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str());
    if get("SCTE35-OUT").is_some() {
        let id = get("ID").ok_or_else(|| SsaiError::InvalidPlaylist("DATERANGE without ID".into()))?;
        let duration = get("PLANNED-DURATION")
            .or_else(|| get("DURATION"))
            .ok_or_else(|| SsaiError::InvalidPlaylist(format!("DATERANGE {id} without duration")))?;
        return Ok(Entry::CueOut { id: id.into(), duration: parse_number(duration)?, raw: raw.into() });
    }
    if get("SCTE35-IN").is_some() {
        return Ok(Entry::CueIn { raw: raw.into() });
    }
    Ok(Entry::Tag(raw.into()))
}

fn split_attributes(attrs: &str) -> Vec<(String, String)> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in attrs.chars() {
        match c {
            '"' => quoted = !quoted,
            ',' if !quoted => parts.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    parts.push(current);
    parts
        .into_iter()
        .filter_map(|p| p.split_once('=').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
        .collect()
}
