use subtitle_service::error::SubError;
use subtitle_service::time::{format_vtt, parse_ass_time};

#[test]
fn ass_centiseconds_become_vtt_milliseconds_without_drift() {
    let ms = parse_ass_time("0:00:01.50").unwrap();
    assert_eq!(ms, 1500, "ASS .50 is fifty centiseconds, reading it as 50ms would desync every cue");
    assert_eq!(format_vtt(ms), "00:00:01.500");
}

#[test]
fn hours_beyond_nine_still_round_trip_for_long_features() {
    let ms = parse_ass_time("12:34:56.78").unwrap();
    assert_eq!(format_vtt(ms), "12:34:56.780");
}

#[test]
fn fractional_precision_is_normalised_to_milliseconds() {
    assert_eq!(parse_ass_time("0:00:02.5").unwrap(), 2500);
    assert_eq!(parse_ass_time("0:00:02.505").unwrap(), 2505);
    assert_eq!(parse_ass_time("0:00:02").unwrap(), 2000);
}

#[test]
fn vtt_timestamps_always_have_two_digit_hours_because_players_reject_short_forms_in_some_engines() {
    assert_eq!(format_vtt(0), "00:00:00.000");
    assert_eq!(format_vtt(61_001), "00:01:01.001");
}

#[test]
fn malformed_timestamps_fail_loudly_instead_of_becoming_zero() {
    for bad in ["", "1:2", "0:61:00.00", "0:00:75.00", "a:00:00.00", "0:00:01.", "0:00:01.x"] {
        assert_eq!(parse_ass_time(bad), Err(SubError::Timestamp(bad.to_string())), "{bad}");
    }
}
