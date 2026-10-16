mod common;

use ssai::playlist::{Entry, Playlist};

#[test]
fn cue_out_and_daterange_both_become_breaks_with_their_planned_duration() {
    let playlist = Playlist::parse(common::CONTENT).unwrap();
    let cues: Vec<(String, f64)> = playlist
        .entries
        .iter()
        .filter_map(|e| match e {
            Entry::CueOut { id, duration, .. } => Some((id.clone(), *duration)),
            _ => None,
        })
        .collect();
    assert_eq!(cues, vec![("break-1".to_string(), 12.0), ("ad-break-2".to_string(), 8.0)]);
}

#[test]
fn render_round_trips_so_unrelated_tags_survive_stitching() {
    let playlist = Playlist::parse(common::CONTENT).unwrap();
    let again = Playlist::parse(&playlist.render()).unwrap();
    assert_eq!(playlist, again);
    assert!(playlist.ended);
    assert_eq!(playlist.segments().count(), 15);
}

#[test]
fn target_duration_grows_when_a_longer_segment_is_inserted() {
    let mut playlist = Playlist::parse(common::CONTENT).unwrap();
    playlist.entries.push(Entry::Segment { duration: 4.5, uri: "long.ts".into() });
    assert!(playlist.render().contains("#EXT-X-TARGETDURATION:5\n"), "players reject segments longer than the target duration");
}

#[test]
fn cue_out_accepts_bare_seconds() {
    let playlist = Playlist::parse("#EXTM3U\n#EXT-X-CUE-OUT:30\n").unwrap();
    assert!(matches!(&playlist.entries[0], Entry::CueOut { duration, .. } if *duration == 30.0));
}

#[test]
fn malformed_playlists_are_rejected() {
    assert!(Playlist::parse("#EXTINF:2.0,\na.ts\n").is_err());
    assert!(Playlist::parse("#EXTM3U\n#EXTINF:abc,\na.ts\n").is_err());
    assert!(Playlist::parse("#EXTM3U\n#EXTINF:2.0,\n").is_err());
    assert!(Playlist::parse("#EXTM3U\n#EXT-X-DATERANGE:ID=\"x\",SCTE35-OUT=0xFC\n").is_err());
}
