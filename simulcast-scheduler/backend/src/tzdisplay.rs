use crate::error::AppError;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalTime {
    pub timezone: String,
    pub local: String,
    pub abbrev: String,
    pub utc_offset: String,
}

pub fn local_time(instant: DateTime<Utc>, timezone: &str) -> Result<LocalTime, AppError> {
    let tz: Tz = timezone
        .parse()
        .map_err(|_| AppError::UnknownTimeZone(timezone.to_string()))?;
    let local = instant.with_timezone(&tz);
    Ok(LocalTime {
        timezone: timezone.to_string(),
        local: local.format("%Y-%m-%d %H:%M:%S").to_string(),
        abbrev: local.format("%Z").to_string(),
        utc_offset: local.format("%:z").to_string(),
    })
}
