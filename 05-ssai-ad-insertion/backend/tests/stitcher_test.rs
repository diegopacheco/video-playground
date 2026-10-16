mod common;

use ssai::playlist::{Entry, Playlist};
use ssai::users::{Tier, UserProfile};

fn free_user(region: &str) -> UserProfile {
    UserProfile::new("alice", "Alice", region, Tier::Free)
}

fn is_ad(uri: &str) -> bool {
    uri.starts_with("/api/track/") || uri.starts_with("/media/ads/")
}

fn ad_index(uri: &str) -> usize {
    let file = uri.rsplit('/').next().unwrap_or_default();
    file.trim_end_matches(".ts").trim_start_matches("seg_").parse().unwrap_or(usize::MAX)
}

fn creative_of(uri: &str) -> String {
    let parts: Vec<&str> = uri.split('/').collect();
    if uri.starts_with("/api/track/") { parts[6].to_string() } else { parts[3].to_string() }
}

fn segment_pairs(playlist: &Playlist) -> Vec<(String, bool, String)> {
    let mut pairs = Vec::new();
    let mut previous: Option<String> = None;
    let mut discontinuity = false;
    for entry in &playlist.entries {
        match entry {
            Entry::Discontinuity => discontinuity = true,
            Entry::Segment { uri, .. } => {
                if let Some(prev) = previous.take() {
                    pairs.push((prev, discontinuity, uri.clone()));
                }
                previous = Some(uri.clone());
                discontinuity = false;
            }
            _ => {}
        }
    }
    pairs
}

fn ad_seconds_between(playlist: &Playlist) -> Vec<f64> {
    let mut runs = Vec::new();
    let mut current = 0.0;
    for (duration, uri) in playlist.segments() {
        if is_ad(uri) {
            current += duration;
        } else if current > 0.0 {
            runs.push(current);
            current = 0.0;
        }
    }
    if current > 0.0 {
        runs.push(current);
    }
    runs
}

#[test]
fn discontinuity_marks_every_timestamp_reset_and_nothing_else() {
    let content = Playlist::parse(common::CONTENT).unwrap();
    let stitched = common::stitcher().stitch(&content, &free_user("us"), "s1").unwrap();
    let pairs = segment_pairs(&stitched.playlist);
    assert!(pairs.iter().any(|(a, _, b)| !is_ad(a) && is_ad(b)), "break must contain ads");
    for (prev, disc, next) in pairs {
        let expected = match (is_ad(&prev), is_ad(&next)) {
            (false, false) => false,
            (false, true) | (true, false) => true,
            (true, true) => ad_index(&next) == 0,
        };
        assert_eq!(disc, expected, "between {prev} and {next}: each creative is its own encode so its PTS restart needs a discontinuity, content-to-content must stay seamless");
    }
}

#[test]
fn discontinuity_closes_the_pod_before_content_resumes() {
    let content = Playlist::parse(common::CONTENT).unwrap();
    let stitched = common::stitcher().stitch(&content, &free_user("us"), "s1").unwrap();
    let resumed = stitched
        .playlist
        .entries
        .iter()
        .position(|e| matches!(e, Entry::Segment { uri, .. } if uri.ends_with("seg_008.ts")))
        .unwrap();
    let before: Vec<&Entry> = stitched.playlist.entries[..resumed].iter().rev().take(2).collect();
    assert!(matches!(before[0], Entry::CueIn { .. }));
    assert_eq!(before[1], &Entry::Discontinuity, "the player must reset its decoder before the original content timeline comes back");
}

#[test]
fn ads_fill_each_break_exactly_so_the_content_timeline_does_not_drift() {
    let content = Playlist::parse(common::CONTENT).unwrap();
    for region in ["us", "br", "eu"] {
        let stitched = common::stitcher().stitch(&content, &free_user(region), "s1").unwrap();
        assert_eq!(ad_seconds_between(&stitched.playlist), vec![12.0, 8.0], "region {region}");
        for brk in &stitched.breaks {
            assert!((brk.filled - brk.planned).abs() < 1e-6, "{} filled {} of {}", brk.id, brk.filled, brk.planned);
            assert!((brk.replaced - brk.planned).abs() < 1e-6);
        }
        assert!((stitched.playlist.total_duration() - content.total_duration()).abs() < 1e-6);
    }
}

#[test]
fn original_break_content_never_leaks_into_an_ad_supported_stream() {
    let content = Playlist::parse(common::CONTENT).unwrap();
    let stitched = common::stitcher().stitch(&content, &free_user("us"), "s1").unwrap();
    let uris: Vec<&str> = stitched.playlist.segments().map(|(_, u)| u).collect();
    for replaced in ["seg_002.ts", "seg_005.ts", "seg_007.ts", "seg_010.ts", "seg_013.ts"] {
        assert!(!uris.iter().any(|u| u.ends_with(&format!("/content/{replaced}"))), "{replaced} should be replaced");
    }
    assert!(uris.contains(&"/media/content/seg_014.ts"), "daterange break ends after its planned duration");
}

#[test]
fn premium_users_bypass_ads_and_get_the_untouched_content() {
    let content = Playlist::parse(common::CONTENT).unwrap();
    let premium = UserProfile::new("diana", "Diana", "us", Tier::Premium);
    let stitched = common::stitcher().stitch(&content, &premium, "s1").unwrap();
    assert!(!stitched.playlist.entries.contains(&Entry::Discontinuity));
    let uris: Vec<String> = stitched.playlist.segments().map(|(_, u)| u.to_string()).collect();
    let expected: Vec<String> = content.segments().map(|(_, u)| format!("/media/content/{u}")).collect();
    assert_eq!(uris, expected);
    assert!(stitched.breaks.iter().all(|b| b.creatives.is_empty()));
}

#[test]
fn creatives_only_reach_their_targeted_regions() {
    let content = Playlist::parse(common::CONTENT).unwrap();
    let br = common::stitcher().stitch(&content, &free_user("br"), "s1").unwrap();
    let us = common::stitcher().stitch(&content, &free_user("us"), "s1").unwrap();
    let ads = |p: &Playlist| p.segments().filter(|(_, u)| is_ad(u)).map(|(_, u)| creative_of(u)).collect::<Vec<_>>();
    assert!(!ads(&br.playlist).contains(&"globex".to_string()));
    assert!(ads(&br.playlist).contains(&"initech".to_string()));
    assert!(!ads(&us.playlist).contains(&"initech".to_string()));
}

#[test]
fn tracked_ad_uris_carry_user_session_and_break_for_beaconing() {
    let content = Playlist::parse(common::CONTENT).unwrap();
    let stitched = common::stitcher().stitch(&content, &free_user("us"), "sess-9").unwrap();
    let tracked: Vec<&str> = stitched.playlist.segments().map(|(_, u)| u).filter(|u| u.starts_with("/api/track/")).collect();
    assert!(!tracked.is_empty());
    assert!(tracked.iter().all(|u| u.starts_with("/api/track/alice/sess-9/break-1/") || u.starts_with("/api/track/alice/sess-9/ad-break-2/")));
    let slate: Vec<&str> = stitched.playlist.segments().map(|(_, u)| u).filter(|u| u.starts_with("/media/ads/slate/")).collect();
    assert!(slate.iter().all(|u| !u.contains("/api/track/")), "slate filler is not an ad and must not fire beacons");
}

#[test]
fn nested_cue_out_is_rejected() {
    let broken = "#EXTM3U\n#EXT-X-CUE-OUT:4\n#EXTINF:2.0,\na.ts\n#EXT-X-CUE-OUT:4\n#EXTINF:2.0,\nb.ts\n";
    let content = Playlist::parse(broken).unwrap();
    assert!(common::stitcher().stitch(&content, &free_user("us"), "s1").is_err());
}
