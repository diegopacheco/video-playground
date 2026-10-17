use serde::{Deserialize, Serialize};

pub const FINISHED_RATIO: f64 = 0.9;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playhead {
    pub user_id: String,
    pub profile_id: String,
    pub device_id: String,
    pub title_id: String,
    pub episode_id: String,
    pub position_secs: u32,
    pub duration_secs: u32,
    pub client_ts_ms: i64,
}

impl Playhead {
    pub fn key(&self) -> (String, String) {
        (self.profile_id.clone(), self.title_id.clone())
    }

    pub fn progress(&self) -> f64 {
        if self.duration_secs == 0 {
            return 0.0;
        }
        f64::from(self.position_secs) / f64::from(self.duration_secs)
    }

    pub fn is_finished(&self) -> bool {
        self.progress() > FINISHED_RATIO
    }

    pub fn wins_over(&self, other: &Playhead) -> bool {
        self.client_ts_ms > other.client_ts_ms
    }

    pub fn validate(&self) -> Result<(), String> {
        let ids = [
            ("user_id", &self.user_id),
            ("profile_id", &self.profile_id),
            ("device_id", &self.device_id),
            ("title_id", &self.title_id),
            ("episode_id", &self.episode_id),
        ];
        if let Some((name, _)) = ids.iter().find(|(_, v)| v.trim().is_empty()) {
            return Err(format!("{name} must not be empty"));
        }
        if self.duration_secs == 0 {
            return Err("duration_secs must be greater than zero".into());
        }
        if self.position_secs > self.duration_secs {
            return Err("position_secs must not exceed duration_secs".into());
        }
        if self.client_ts_ms <= 0 {
            return Err("client_ts_ms must be positive".into());
        }
        Ok(())
    }
}

#[cfg(test)]
pub fn playhead(profile: &str, title: &str, device: &str, position: u32, ts: i64) -> Playhead {
    Playhead {
        user_id: "u1".into(),
        profile_id: profile.into(),
        device_id: device.into(),
        title_id: title.into(),
        episode_id: "s1e1".into(),
        position_secs: position,
        duration_secs: 1000,
        client_ts_ms: ts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_ninety_percent_still_counts_as_in_progress() {
        assert!(!playhead("p", "t", "d", 900, 1).is_finished());
        assert!(playhead("p", "t", "d", 901, 1).is_finished());
    }

    #[test]
    fn rejects_position_beyond_duration_so_progress_never_exceeds_one() {
        assert!(playhead("p", "t", "d", 1001, 1).validate().is_err());
    }

    #[test]
    fn rejects_missing_ids_so_keys_are_never_ambiguous() {
        let mut p = playhead("p", "t", "d", 1, 1);
        p.title_id = " ".into();
        assert!(p.validate().is_err());
    }
}
