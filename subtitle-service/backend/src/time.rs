use crate::error::SubError;

pub fn parse_ass_time(input: &str) -> Result<u64, SubError> {
    let bad = || SubError::Timestamp(input.to_string());
    let trimmed = input.trim();
    let mut parts = trimmed.split(':');
    let (h, m, rest) = match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(h), Some(m), Some(rest), None) => (h, m, rest),
        _ => return Err(bad()),
    };
    let (s, frac) = rest.split_once('.').unwrap_or((rest, "0"));
    let hours: u64 = digits(h).ok_or_else(bad)?;
    let minutes: u64 = digits(m).ok_or_else(bad)?;
    let seconds: u64 = digits(s).ok_or_else(bad)?;
    if minutes > 59 || seconds > 59 || frac.is_empty() || !frac.bytes().all(|b| b.is_ascii_digit()) {
        return Err(bad());
    }
    let millis = frac_to_millis(frac);
    Ok(((hours * 60 + minutes) * 60 + seconds) * 1000 + millis)
}

fn digits(s: &str) -> Option<u64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn frac_to_millis(frac: &str) -> u64 {
    let mut padded: String = frac.chars().take(3).collect();
    while padded.len() < 3 {
        padded.push('0');
    }
    padded.parse().unwrap_or(0)
}

pub fn format_vtt(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let minutes = (ms / 60_000) % 60;
    let seconds = (ms / 1000) % 60;
    let millis = ms % 1000;
    format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
}
