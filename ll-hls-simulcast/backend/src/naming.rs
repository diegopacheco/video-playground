use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    LowLatency,
    Regular,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Playlist(Mode),
    Init,
    Segment(u64),
    Part(u64, usize),
}

pub const INIT_URI: &str = "init.mp4";

pub fn segment_uri(msn: u64) -> String {
    format!("seg{msn}.m4s")
}

pub fn part_uri(msn: u64, index: usize) -> String {
    format!("part{msn}.{index}.m4s")
}

pub fn parse(file: &str) -> Result<Resource> {
    match file {
        "ll.m3u8" => return Ok(Resource::Playlist(Mode::LowLatency)),
        "hls.m3u8" => return Ok(Resource::Playlist(Mode::Regular)),
        INIT_URI => return Ok(Resource::Init),
        _ => {}
    }
    let stem = file.strip_suffix(".m4s").ok_or(Error::NotFound)?;
    if let Some(rest) = stem.strip_prefix("part") {
        let (msn, index) = rest.split_once('.').ok_or(Error::NotFound)?;
        return Ok(Resource::Part(
            msn.parse().map_err(|_| Error::NotFound)?,
            index.parse().map_err(|_| Error::NotFound)?,
        ));
    }
    if let Some(msn) = stem.strip_prefix("seg") {
        return Ok(Resource::Segment(msn.parse().map_err(|_| Error::NotFound)?));
    }
    Err(Error::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uris_written_into_playlists_resolve_back_to_the_same_media() {
        assert_eq!(parse(&segment_uri(42)).unwrap(), Resource::Segment(42));
        assert_eq!(parse(&part_uri(42, 7)).unwrap(), Resource::Part(42, 7));
        assert_eq!(parse(INIT_URI).unwrap(), Resource::Init);
    }

    #[test]
    fn both_simulcast_playlists_are_addressable() {
        assert_eq!(parse("ll.m3u8").unwrap(), Resource::Playlist(Mode::LowLatency));
        assert_eq!(parse("hls.m3u8").unwrap(), Resource::Playlist(Mode::Regular));
    }

    #[test]
    fn unknown_or_malformed_names_are_not_found_rather_than_misrouted() {
        assert!(parse("seg.m4s").is_err());
        assert!(parse("part3.m4s").is_err());
        assert!(parse("../etc/passwd").is_err());
        assert!(parse("seg-1.m4s").is_err());
    }
}
