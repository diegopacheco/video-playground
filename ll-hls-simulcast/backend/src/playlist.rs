use crate::clock::iso8601;
use crate::live::{Segment, Window};
use crate::naming::{INIT_URI, part_uri, segment_uri};

pub trait PlaylistBuilder: Send + Sync {
    fn build(&self, window: &Window) -> String;
}

pub struct LowLatencyPlaylist {
    pub part_hold_back_factor: f64,
    pub part_window_targets: f64,
}

pub struct RegularPlaylist {
    pub hold_back_segments: f64,
}

fn target_duration(window: &Window) -> u64 {
    let longest = window.complete_segments().map(Segment::duration).fold(0.0, f64::max);
    longest.round().max(window.config.segment_target.ceil()) as u64
}

fn header(window: &Window, out: &mut String) {
    out.push_str("#EXTM3U\n#EXT-X-VERSION:6\n");
    out.push_str(&format!("#EXT-X-TARGETDURATION:{}\n", target_duration(window)));
}

fn media_sequence(window: &Window, segments: &[&Segment], out: &mut String) {
    let first = segments.first().map(|s| s.msn).unwrap_or(0);
    out.push_str(&format!("#EXT-X-MEDIA-SEQUENCE:{first}\n"));
    if window.init.is_some() {
        out.push_str(&format!("#EXT-X-MAP:URI=\"{INIT_URI}\"\n"));
    }
}

fn segment_lines(segment: &Segment, out: &mut String) {
    out.push_str(&format!("#EXTINF:{:.5},\n{}\n", segment.duration(), segment_uri(segment.msn)));
}

fn program_date(segment: &Segment, out: &mut String) {
    out.push_str(&format!("#EXT-X-PROGRAM-DATE-TIME:{}\n", iso8601(segment.program_date_ms)));
}

impl PlaylistBuilder for LowLatencyPlaylist {
    fn build(&self, window: &Window) -> String {
        let mut out = String::new();
        let part_target = window.config.part_target;
        header(window, &mut out);
        out.push_str(&format!("#EXT-X-PART-INF:PART-TARGET={part_target:.5}\n"));
        out.push_str(&format!(
            "#EXT-X-SERVER-CONTROL:CAN-BLOCK-RELOAD=YES,PART-HOLD-BACK={:.5}\n",
            part_target * self.part_hold_back_factor
        ));
        let segments: Vec<&Segment> = window.segments.iter().collect();
        media_sequence(window, &segments, &mut out);
        let part_horizon = self.part_window_targets * target_duration(window) as f64;
        let mut remaining: f64 = segments.iter().map(|s| s.duration()).sum();
        for segment in &segments {
            program_date(segment, &mut out);
            if remaining <= part_horizon + 1e-9 || !segment.complete {
                for (i, part) in segment.parts.iter().enumerate() {
                    out.push_str(&format!(
                        "#EXT-X-PART:DURATION={:.5},URI=\"{}\"{}\n",
                        part.duration,
                        part_uri(segment.msn, i),
                        if part.independent { ",INDEPENDENT=YES" } else { "" }
                    ));
                }
            }
            remaining -= segment.duration();
            if segment.complete {
                segment_lines(segment, &mut out);
            }
        }
        if let Some(last) = segments.last() {
            let (msn, index) = if last.complete { (last.msn + 1, 0) } else { (last.msn, last.parts.len()) };
            out.push_str(&format!("#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"{}\"\n", part_uri(msn, index)));
        }
        out
    }
}

impl PlaylistBuilder for RegularPlaylist {
    fn build(&self, window: &Window) -> String {
        let mut out = String::new();
        header(window, &mut out);
        out.push_str(&format!(
            "#EXT-X-SERVER-CONTROL:HOLD-BACK={:.5}\n",
            self.hold_back_segments * target_duration(window) as f64
        ));
        let segments: Vec<&Segment> = window.complete_segments().collect();
        media_sequence(window, &segments, &mut out);
        for segment in &segments {
            program_date(segment, &mut out);
            segment_lines(segment, &mut out);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::fixture::{config, feed};
    use axum::body::Bytes;

    fn window(parts: usize) -> Window {
        let mut w = Window::new(config());
        w.init = Some(Bytes::from_static(b"init"));
        feed(&mut w, parts);
        w
    }

    fn ll() -> LowLatencyPlaylist {
        LowLatencyPlaylist { part_hold_back_factor: 3.0, part_window_targets: 3.0 }
    }

    fn regular() -> RegularPlaylist {
        RegularPlaylist { hold_back_segments: 3.0 }
    }

    fn attr(text: &str, tag: &str, name: &str) -> Option<String> {
        let line = text.lines().find(|l| l.starts_with(tag))?;
        line.split([':', ','])
            .find_map(|kv| kv.strip_prefix(&format!("{name}=")))
            .map(|v| v.trim_matches('"').to_string())
    }

    fn values(text: &str, prefix: &str) -> Vec<f64> {
        text.lines()
            .filter_map(|l| l.strip_prefix(prefix))
            .map(|v| v.split([',', '\n']).next().unwrap_or("").parse().unwrap())
            .collect()
    }

    #[test]
    fn playlist_starts_with_extm3u_because_players_reject_anything_else() {
        assert!(ll().build(&window(35)).starts_with("#EXTM3U\n"));
        assert!(regular().build(&window(35)).starts_with("#EXTM3U\n"));
    }

    #[test]
    fn every_segment_rounds_to_at_most_the_target_duration() {
        let text = ll().build(&window(35));
        let target: f64 = text.lines().find_map(|l| l.strip_prefix("#EXT-X-TARGETDURATION:")).unwrap().parse().unwrap();
        for d in values(&text, "#EXTINF:") {
            assert!(d.round() <= target);
        }
    }

    #[test]
    fn every_part_fits_inside_part_target() {
        let text = ll().build(&window(35));
        let target: f64 = attr(&text, "#EXT-X-PART-INF", "PART-TARGET").unwrap().parse().unwrap();
        for d in values(&text, "#EXT-X-PART:DURATION=") {
            assert!(d <= target + 1e-9);
        }
    }

    #[test]
    fn part_hold_back_is_at_least_twice_part_target_as_the_spec_requires() {
        let text = ll().build(&window(35));
        let target: f64 = attr(&text, "#EXT-X-PART-INF", "PART-TARGET").unwrap().parse().unwrap();
        let hold: f64 = attr(&text, "#EXT-X-SERVER-CONTROL", "PART-HOLD-BACK").unwrap().parse().unwrap();
        assert!(hold >= 2.0 * target);
    }

    #[test]
    fn server_advertises_blocking_reload_so_clients_can_use_msn_and_part() {
        let text = ll().build(&window(35));
        assert_eq!(attr(&text, "#EXT-X-SERVER-CONTROL", "CAN-BLOCK-RELOAD").as_deref(), Some("YES"));
    }

    #[test]
    fn preload_hint_names_the_very_next_part_so_clients_can_request_it_early() {
        let text = ll().build(&window(35));
        assert_eq!(attr(&text, "#EXT-X-PRELOAD-HINT", "TYPE").as_deref(), Some("PART"));
        assert_eq!(attr(&text, "#EXT-X-PRELOAD-HINT", "URI"), Some(part_uri(3, 5)));
        assert_eq!(text.lines().last().unwrap(), "#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part3.5.m4s\"");
    }

    #[test]
    fn open_segment_is_published_only_as_parts_until_it_is_complete() {
        let text = ll().build(&window(35));
        assert!(text.contains("part3.4.m4s"));
        assert!(!text.contains("seg3.m4s"));
        assert!(text.contains("seg2.m4s"));
    }

    #[test]
    fn parts_older_than_three_target_durations_are_dropped_to_keep_the_playlist_small() {
        let text = ll().build(&window(60));
        assert!(!text.contains("part0."));
        assert!(text.contains("seg0.m4s"));
        assert!(text.contains("part3.0.m4s"));
    }

    #[test]
    fn independent_parts_are_flagged_so_players_can_join_mid_segment() {
        let text = ll().build(&window(35));
        assert!(text.contains("URI=\"part3.0.m4s\",INDEPENDENT=YES"));
        assert!(!text.contains("URI=\"part3.1.m4s\",INDEPENDENT=YES"));
    }

    #[test]
    fn media_sequence_tracks_the_sliding_window() {
        let text = ll().build(&window(100));
        assert!(text.contains("#EXT-X-MEDIA-SEQUENCE:4\n"));
        assert!(text.contains("seg4.m4s"));
        assert!(!text.contains("seg3.m4s"));
    }

    #[test]
    fn every_segment_carries_program_date_time_for_latency_measurement() {
        let w = window(35);
        let text = ll().build(&w);
        assert_eq!(text.matches("#EXT-X-PROGRAM-DATE-TIME:").count(), w.segments.len());
        assert!(text.contains(&format!("#EXT-X-PROGRAM-DATE-TIME:{}", iso8601(w.segments[1].program_date_ms))));
    }

    #[test]
    fn regular_playlist_lists_only_complete_segments_and_no_low_latency_tags() {
        let text = regular().build(&window(35));
        assert!(!text.contains("#EXT-X-PART"));
        assert!(!text.contains("PRELOAD-HINT"));
        assert!(!text.contains("CAN-BLOCK-RELOAD"));
        assert!(text.contains("seg2.m4s"));
        assert!(!text.contains("seg3.m4s"));
        assert_eq!(values(&text, "#EXTINF:").len(), 3);
    }

    #[test]
    fn regular_hold_back_is_at_least_three_target_durations() {
        let text = regular().build(&window(35));
        let hold: f64 = attr(&text, "#EXT-X-SERVER-CONTROL", "HOLD-BACK").unwrap().parse().unwrap();
        assert!(hold >= 3.0 * 2.0);
    }

    #[test]
    fn empty_window_still_yields_a_valid_header() {
        let text = ll().build(&Window::new(config()));
        assert!(text.starts_with("#EXTM3U\n"));
        assert!(!text.contains("PRELOAD-HINT"));
    }
}
