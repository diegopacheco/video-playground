use std::time::{SystemTime, UNIX_EPOCH};

pub const STRIP_WIDTH: usize = 960;
pub const STRIP_HEIGHT: usize = 120;
const SCALE: usize = 12;
const CELL: usize = 6 * SCALE;
const BACKGROUND: u8 = 16;
const INK: u8 = 235;

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

pub fn iso8601(epoch_ms: i64) -> String {
    let days = epoch_ms.div_euclid(86_400_000);
    let (year, month, day) = civil(days);
    format!("{year:04}-{month:02}-{day:02}T{}Z", time_of_day(epoch_ms))
}

pub fn time_of_day(epoch_ms: i64) -> String {
    let ms = epoch_ms.rem_euclid(86_400_000);
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms % 1000
    )
}

fn glyph(c: char) -> [u8; 7] {
    match c {
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1F, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        ':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        _ => [0; 7],
    }
}

pub fn render_strip(text: &str) -> Vec<u8> {
    let mut frame = vec![BACKGROUND; STRIP_WIDTH * STRIP_HEIGHT];
    let width = text.chars().count() * CELL;
    let left = STRIP_WIDTH.saturating_sub(width) / 2;
    let top = (STRIP_HEIGHT - 7 * SCALE) / 2;
    for (i, c) in text.chars().enumerate() {
        for (row, bits) in glyph(c).iter().enumerate() {
            for col in 0..5 {
                if bits & (0x10 >> col) == 0 {
                    continue;
                }
                let x0 = left + i * CELL + col * SCALE;
                let y0 = top + row * SCALE;
                for y in y0..y0 + SCALE {
                    let start = y * STRIP_WIDTH + x0;
                    if x0 + SCALE <= STRIP_WIDTH {
                        frame[start..start + SCALE].fill(INK);
                    }
                }
            }
        }
    }
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_date_time_is_utc_iso8601_with_millis_as_hls_requires() {
        assert_eq!(iso8601(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(iso8601(1_792_108_800_123), "2026-10-16T00:00:00.123Z");
        assert_eq!(iso8601(951_782_400_000), "2000-02-29T00:00:00.000Z");
    }

    #[test]
    fn burned_in_clock_matches_program_date_time_time_of_day() {
        let ms = 1_792_150_496_789;
        assert!(iso8601(ms).contains(&time_of_day(ms)));
        assert_eq!(time_of_day(ms), "11:34:56.789");
    }

    #[test]
    fn strip_has_exact_rawvideo_size_or_ffmpeg_would_desync_frames() {
        assert_eq!(render_strip("12:34:56.789").len(), STRIP_WIDTH * STRIP_HEIGHT);
    }

    #[test]
    fn different_times_render_different_pixels_so_the_clock_visibly_ticks() {
        assert_ne!(render_strip("12:34:56.789"), render_strip("12:34:56.790"));
        assert!(render_strip("00:00:00.000").contains(&INK));
    }
}
