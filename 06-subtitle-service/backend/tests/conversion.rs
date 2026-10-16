use subtitle_service::ass::{AssParser, SubtitleParser};
use subtitle_service::hls::{Rendition, master_playlist, playlist_duration, subtitle_playlist, with_timestamp_map};
use subtitle_service::webvtt::{convert, cue_settings};

fn script(events: &str) -> String {
    format!(
        "[Script Info]
PlayResX: 1000
PlayResY: 500

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, Bold, Italic, Outline, Shadow, Alignment
Style: Default,Noto Sans,40,&H00FFFFFF,0,0,0,0,2
Style: Sign,Noto Serif,40,&H0000D7FF,-1,0,0,0,7

[Events]
Format: Start, End, Style, Text
{events}"
    )
}

fn vtt(events: &str) -> (String, subtitle_service::webvtt::LossReport) {
    let parsed = AssParser.parse(&script(events)).unwrap();
    let out = convert(&parsed);
    (out.vtt, out.report)
}

#[test]
fn inline_styles_become_webvtt_tags_and_text_is_escaped() {
    let (out, report) = vtt("Dialogue: 0:00:01.00,0:00:02.00,Default,{\\i1}a<b>{\\i0} & {\\b1}c{\\b0}\n");
    assert!(out.contains("<c.sDefault><i>a&lt;b&gt;</i> &amp; <b>c</b></c>"), "{out}");
    assert_eq!(report.fidelity, 1.0);
}

#[test]
fn colour_override_maps_to_a_class_declared_in_the_style_block() {
    let (out, _) = vtt("Dialogue: 0:00:01.00,0:00:02.00,Default,{\\c&H0000FF&}red{\\c} plain\n");
    assert!(out.contains("::cue(.cFF0000) { color: #FF0000; }"));
    assert!(out.contains("<c.cFF0000>red</c> plain"));
}

#[test]
fn style_colour_and_font_are_carried_by_a_style_class() {
    let (out, _) = vtt("Dialogue: 0:00:01.00,0:00:02.00,Sign,x\n");
    assert!(out.contains("::cue(.sSign) { color: #FFD700; font-family: \"Noto Serif\", sans-serif; }"));
    assert!(out.contains("<c.sSign><b>x</b></c>"), "style bold must survive as <b>");
}

#[test]
fn pos_is_converted_to_percent_cue_settings_anchored_by_alignment() {
    let (out, _) = vtt("Dialogue: 0:00:01.00,0:00:02.00,Default,{\\an9\\pos(900,100)}corner\n");
    assert!(out.contains("--> 00:00:02.000 line:20%,start position:90%,line-right align:end"), "{out}");
}

#[test]
fn alignment_without_pos_uses_edge_presets() {
    assert_eq!(cue_settings(2, None, 1, 1), "");
    assert_eq!(cue_settings(8, None, 1, 1), " line:5%,start");
    assert_eq!(cue_settings(4, None, 1, 1), " line:50%,center position:5%,line-left align:start");
}

#[test]
fn karaoke_syllables_become_inline_timestamps_relative_to_the_cue() {
    let (out, report) = vtt("Dialogue: 0:00:01.00,0:00:03.00,Default,{\\k50}Ka{\\k50}ra{\\kf50}o\n");
    assert!(out.contains("Ka<00:00:01.500>ra<00:00:02.000>o"), "{out}");
    assert_eq!(report.kept.get("k"), Some(&2));
    assert_eq!(report.approximated.get("kf"), Some(&1), "sweep fill becomes a step");
}

#[test]
fn unsupported_overrides_are_stripped_and_counted_as_lossy() {
    let (out, report) = vtt("Dialogue: 0:00:01.00,0:00:02.00,Default,{\\frz30\\blur2\\fad(100,100)\\move(0,0,10,10)}spin\n");
    assert!(out.contains("<c.sDefault>spin</c>"));
    assert!(!out.contains("frz"));
    assert_eq!(report.dropped.len(), 3);
    assert_eq!(report.approximated.get("move"), Some(&1));
    assert!(report.fidelity < 0.2, "{}", report.fidelity);
}

#[test]
fn drawing_only_events_are_dropped_and_all_their_tags_count_as_lost() {
    let (out, report) = vtt("Dialogue: 0:00:01.00,0:00:02.00,Default,{\\pos(1,1)\\p1}m 0 0 l 1 1{\\p0}\n");
    assert_eq!(report.cues_in, 1);
    assert_eq!(report.cues_out, 0);
    assert!(!out.contains("-->"));
    assert_eq!(report.dropped.get("pos"), Some(&1));
}

#[test]
fn cues_are_emitted_in_start_order_even_when_the_file_is_not() {
    let (out, _) = vtt("Dialogue: 0:00:05.00,0:00:06.00,Default,second\nDialogue: 0:00:01.00,0:00:02.00,Default,first\n");
    assert!(out.find("first").unwrap() < out.find("second").unwrap());
}

#[test]
fn hls_master_declares_subtitle_group_and_links_it_to_the_video_variant() {
    let video = "#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-STREAM-INF:BANDWIDTH=900000,RESOLUTION=1280x720,CODECS=\"avc1.64001f\"\nvideo.m3u8\n";
    let renditions = [
        Rendition { locale: "en-US".into(), name: "English".into(), default: true },
        Rendition { locale: "ja-JP".into(), name: "日本語".into(), default: false },
    ];
    let master = master_playlist(video, "/media/hls/", &renditions).unwrap();
    assert!(master.contains("#EXT-X-MEDIA:TYPE=SUBTITLES,GROUP-ID=\"subs\",NAME=\"English\",LANGUAGE=\"en-US\",DEFAULT=YES"));
    assert!(master.contains("LANGUAGE=\"ja-JP\",DEFAULT=NO,AUTOSELECT=YES,FORCED=NO,URI=\"subs/ja-JP/index.m3u8\""));
    assert!(master.contains("CODECS=\"avc1.64001f\",SUBTITLES=\"subs\"\n/media/hls/video.m3u8\n"));
    assert!(master_playlist("#EXTM3U\n", "/", &renditions).is_err());
}

#[test]
fn subtitle_playlist_covers_the_whole_video_duration() {
    let duration = playlist_duration("#EXTINF:4.0,\na.m4s\n#EXTINF:4.0,\nb.m4s\n#EXTINF:2.5,\nc.m4s\n").unwrap();
    assert_eq!(duration, 10.5);
    let playlist = subtitle_playlist(duration);
    assert!(playlist.contains("#EXT-X-TARGETDURATION:11\n"), "target must be >= segment length");
    assert!(playlist.contains("#EXTINF:10.500,\nsubs.vtt\n#EXT-X-ENDLIST"));
}

#[test]
fn hls_vtt_segments_carry_a_timestamp_map_for_media_alignment() {
    let mapped = with_timestamp_map("WEBVTT\n\n1\n00:00:01.000 --> 00:00:02.000\nx\n");
    assert!(mapped.starts_with("WEBVTT\nX-TIMESTAMP-MAP=MPEGTS:0,LOCAL:00:00:00.000\n\n1\n"));
}
