use crate::error::{AppError, Result};
use std::path::{Component, Path, PathBuf};

pub fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "m3u8" => "application/vnd.apple.mpegurl",
        "mpd" => "application/dash+xml",
        "m4s" => "video/iso.segment",
        "mp4" => "video/mp4",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

pub fn safe_join(root: &Path, requested: &str) -> Result<PathBuf> {
    let relative = Path::new(requested);
    if requested.is_empty() || !relative.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err(AppError::InvalidPath(requested.to_string()));
    }
    Ok(root.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaming_files_get_the_mime_types_players_and_cdns_expect() {
        assert_eq!(content_type(Path::new("master.m3u8")), "application/vnd.apple.mpegurl");
        assert_eq!(content_type(Path::new("manifest.mpd")), "application/dash+xml");
        assert_eq!(content_type(Path::new("720p/seg_00001.m4s")), "video/iso.segment");
        assert_eq!(content_type(Path::new("720p/init.mp4")), "video/mp4");
        assert_eq!(content_type(Path::new("x.bin")), "application/octet-stream");
    }

    #[test]
    fn traversal_outside_the_output_folder_is_refused() {
        let root = Path::new("/srv/out");
        assert!(safe_join(root, "../secret").is_err());
        assert!(safe_join(root, "/etc/passwd").is_err());
        assert!(safe_join(root, "720p/../../x").is_err());
        assert_eq!(safe_join(root, "720p/init.mp4").unwrap(), Path::new("/srv/out/720p/init.mp4"));
    }
}
