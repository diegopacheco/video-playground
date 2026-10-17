use serde::{Deserialize, Serialize};

use crate::HerdError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SimConfig {
    pub users: u32,
    pub seed: u64,
    pub press_window_ms: u64,
    pub api_rps: u64,
    pub origin_rps: u64,
    pub admission_rps: u64,
    pub edges: u32,
    pub renditions: u32,
    pub api_latency_ms: u64,
    pub api_reject_ms: u64,
    pub edge_latency_ms: u64,
    pub origin_latency_ms: u64,
    pub origin_reject_ms: u64,
    pub jitter_ms: u64,
    pub max_attempts: u32,
    pub give_up_ms: u64,
    pub backoff_base_ms: u64,
    pub backoff_cap_ms: u64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            users: 1_000_000,
            seed: 42,
            press_window_ms: 2_000,
            api_rps: 25_000,
            origin_rps: 2_000,
            admission_rps: 20_000,
            edges: 20,
            renditions: 4,
            api_latency_ms: 40,
            api_reject_ms: 20,
            edge_latency_ms: 15,
            origin_latency_ms: 120,
            origin_reject_ms: 30,
            jitter_ms: 20,
            max_attempts: 10,
            give_up_ms: 300_000,
            backoff_base_ms: 250,
            backoff_cap_ms: 16_000,
        }
    }
}

pub const MAX_USERS: u32 = 5_000_000;

impl SimConfig {
    pub fn validate(&self) -> Result<(), HerdError> {
        let checks = [
            (self.users == 0 || self.users > MAX_USERS, "users must be between 1 and 5000000"),
            (self.api_rps == 0, "api_rps must be positive"),
            (self.origin_rps == 0, "origin_rps must be positive"),
            (self.admission_rps == 0, "admission_rps must be positive"),
            (self.edges == 0 || self.edges > 1_000, "edges must be between 1 and 1000"),
            (self.renditions == 0 || self.renditions > 16, "renditions must be between 1 and 16"),
            (self.max_attempts == 0, "max_attempts must be positive"),
            (self.give_up_ms == 0, "give_up_ms must be positive"),
            (self.backoff_base_ms == 0, "backoff_base_ms must be positive"),
            (self.backoff_cap_ms < self.backoff_base_ms, "backoff_cap_ms must be >= backoff_base_ms"),
        ];
        match checks.iter().find(|(bad, _)| *bad) {
            Some((_, msg)) => Err(HerdError::InvalidConfig((*msg).to_string())),
            None => Ok(()),
        }
    }

    pub fn objects_per_edge(&self) -> u32 {
        1 + self.renditions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_models_a_million_viewers_at_midnight() {
        let cfg = SimConfig::default();
        assert_eq!(cfg.users, 1_000_000);
        assert!(cfg.validate().is_ok());
        assert!(cfg.admission_rps < cfg.api_rps, "waiting room must admit below what the API sustains");
    }

    #[test]
    fn nonsense_configs_are_refused_instead_of_simulated() {
        let zero = SimConfig { users: 0, ..SimConfig::default() };
        assert!(zero.validate().is_err());
        let cap = SimConfig { backoff_cap_ms: 10, backoff_base_ms: 100, ..SimConfig::default() };
        assert!(cap.validate().is_err());
    }
}
