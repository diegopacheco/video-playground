use subtitle_service::ass::{AssParser, ScriptKind, SubtitleParser};
use subtitle_service::color::Color;
use subtitle_service::error::SubError;

const ASS: &str = "\u{feff}[Script Info]
Title: Test
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, Bold, Italic, Alignment
Style: Default,Noto Sans,48,&H0000D7FF,-1,0,8

[Events]
Format: Start, End, Style, Text
Comment: 0:00:00.00,0:00:05.00,Default,ignored note
Dialogue: 0:00:01.00,0:00:02.00,Default,Hello, world, with commas
";

#[test]
fn format_line_controls_field_order_so_reordered_files_still_parse() {
    let script = AssParser.parse(ASS).unwrap();
    assert_eq!(script.info.play_res_x, 1920);
    let style = &script.styles[0];
    assert_eq!(style.font_name, "Noto Sans");
    assert!(style.bold);
    assert_eq!(style.alignment, 8);
    assert_eq!(style.primary, Color { r: 255, g: 215, b: 0, alpha: 0 });
}

#[test]
fn text_field_keeps_its_commas_because_it_is_always_the_last_column() {
    let script = AssParser.parse(ASS).unwrap();
    assert_eq!(script.events.len(), 1, "Comment lines must not become cues");
    assert_eq!(script.events[0].text, "Hello, world, with commas");
    assert_eq!(script.events[0].start, 1000);
}

#[test]
fn legacy_ssa_alignment_is_remapped_to_numpad_layout() {
    let ssa = "[Script Info]
ScriptType: v4.00

[V4 Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, Alignment
Style: Default,Arial,20,16777215,6

[Events]
Format: Marked, Start, End, Style, Text
Dialogue: Marked=0,0:00:00.00,0:00:01.00,Default,hi
";
    let script = AssParser.parse(ssa).unwrap();
    assert_eq!(script.info.kind, ScriptKind::Ssa);
    assert_eq!(script.styles[0].alignment, 8, "SSA 6 is top-center, numpad 8");
    assert_eq!(script.styles[0].primary, Color::WHITE, "SSA uses decimal BGR colors");
    assert_eq!(script.info.play_res_x, 384, "libass default canvas when PlayRes is missing");
}

#[test]
fn unknown_style_falls_back_to_default_like_libass() {
    let script = AssParser.parse(ASS).unwrap();
    assert_eq!(script.style("Nope").name, "Default");
}

#[test]
fn broken_lines_report_the_line_number_so_authors_can_fix_them() {
    let broken = "[Script Info]\nTitle: x\n\n[Events]\nFormat: Start, End, Style, Text\nDialogue: 0:00:01.00,Default\n";
    match AssParser.parse(broken) {
        Err(SubError::Parse { line, .. }) => assert_eq!(line, 6),
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn cue_ending_before_it_starts_is_rejected_since_webvtt_cannot_express_it() {
    let bad = "[Script Info]\n\n[Events]\nFormat: Start, End, Style, Text\nDialogue: 0:00:02.00,0:00:01.00,Default,x\n";
    assert!(matches!(AssParser.parse(bad), Err(SubError::Parse { line: 5, .. })));
}

#[test]
fn invalid_colors_and_missing_script_info_are_errors() {
    assert!(Color::parse("&HZZ&").is_err());
    assert!(AssParser.parse("[Events]\n").is_err());
}

#[test]
fn every_committed_locale_file_parses() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../subs");
    for locale in ["en-US", "pt-BR", "es-419", "ja-JP"] {
        let text = std::fs::read_to_string(format!("{dir}/{locale}.ass")).unwrap();
        let script = AssParser.parse(&text).unwrap();
        assert_eq!(script.styles.len(), 4, "{locale}");
        assert!(script.events.len() >= 8, "{locale}");
    }
}
