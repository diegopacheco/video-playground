use crate::HerdError;
use crate::rng::Rng;
use crate::sim::SimConfig;

pub trait Strategy: Send + Sync {
    fn name(&self) -> &'static str;

    fn prewarm_cdn(&self) -> bool {
        false
    }

    fn uses_waiting_room(&self) -> bool {
        false
    }

    fn retry_delay_ms(&self, failures: u32, rng: &mut Rng) -> u64;
}

pub struct Naive;

impl Strategy for Naive {
    fn name(&self) -> &'static str {
        "naive"
    }

    fn retry_delay_ms(&self, _failures: u32, _rng: &mut Rng) -> u64 {
        0
    }
}

pub struct JitteredBackoff {
    pub base_ms: u64,
    pub cap_ms: u64,
}

impl JitteredBackoff {
    pub fn window_ms(&self, failures: u32) -> u64 {
        let shift = failures.saturating_sub(1).min(32);
        self.base_ms.saturating_mul(1u64 << shift).min(self.cap_ms)
    }
}

impl Strategy for JitteredBackoff {
    fn name(&self) -> &'static str {
        "jittered-backoff"
    }

    fn retry_delay_ms(&self, failures: u32, rng: &mut Rng) -> u64 {
        rng.below(self.window_ms(failures) + 1)
    }
}

pub struct CdnPrewarm;

impl Strategy for CdnPrewarm {
    fn name(&self) -> &'static str {
        "cdn-prewarm"
    }

    fn prewarm_cdn(&self) -> bool {
        true
    }

    fn retry_delay_ms(&self, failures: u32, rng: &mut Rng) -> u64 {
        Naive.retry_delay_ms(failures, rng)
    }
}

pub struct WaitingRoom;

impl Strategy for WaitingRoom {
    fn name(&self) -> &'static str {
        "waiting-room"
    }

    fn uses_waiting_room(&self) -> bool {
        true
    }

    fn retry_delay_ms(&self, failures: u32, rng: &mut Rng) -> u64 {
        Naive.retry_delay_ms(failures, rng)
    }
}

pub struct Combined {
    pub backoff: JitteredBackoff,
}

impl Strategy for Combined {
    fn name(&self) -> &'static str {
        "combined"
    }

    fn prewarm_cdn(&self) -> bool {
        CdnPrewarm.prewarm_cdn()
    }

    fn uses_waiting_room(&self) -> bool {
        WaitingRoom.uses_waiting_room()
    }

    fn retry_delay_ms(&self, failures: u32, rng: &mut Rng) -> u64 {
        self.backoff.retry_delay_ms(failures, rng)
    }
}

pub fn all_strategies(cfg: &SimConfig) -> Vec<Box<dyn Strategy>> {
    let backoff = || JitteredBackoff { base_ms: cfg.backoff_base_ms, cap_ms: cfg.backoff_cap_ms };
    vec![
        Box::new(Naive),
        Box::new(backoff()),
        Box::new(CdnPrewarm),
        Box::new(WaitingRoom),
        Box::new(Combined { backoff: backoff() }),
    ]
}

pub fn strategy_by_name(name: &str, cfg: &SimConfig) -> Result<Box<dyn Strategy>, HerdError> {
    all_strategies(cfg)
        .into_iter()
        .find(|s| s.name() == name)
        .ok_or_else(|| HerdError::UnknownStrategy(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_window_doubles_then_stops_at_the_cap() {
        let b = JitteredBackoff { base_ms: 250, cap_ms: 16_000 };
        let windows: Vec<u64> = (1..=9).map(|f| b.window_ms(f)).collect();
        assert_eq!(windows, vec![250, 500, 1_000, 2_000, 4_000, 8_000, 16_000, 16_000, 16_000]);
    }

    #[test]
    fn jitter_spreads_clients_who_failed_together() {
        let b = JitteredBackoff { base_ms: 250, cap_ms: 16_000 };
        let mut rng = Rng::new(3);
        let delays: Vec<u64> = (0..1_000).map(|_| b.retry_delay_ms(5, &mut rng)).collect();
        assert!(delays.iter().all(|&d| d <= 4_000));
        let distinct: std::collections::HashSet<u64> = delays.iter().copied().collect();
        assert!(distinct.len() > 500, "identical delays would re-synchronise the herd");
    }

    #[test]
    fn naive_retries_immediately() {
        let mut rng = Rng::new(1);
        assert_eq!(Naive.retry_delay_ms(3, &mut rng), 0);
    }

    #[test]
    fn combined_turns_every_protection_on() {
        let cfg = SimConfig::default();
        let c = strategy_by_name("combined", &cfg).unwrap();
        assert!(c.prewarm_cdn() && c.uses_waiting_room());
        assert!(strategy_by_name("nope", &cfg).is_err());
        assert_eq!(all_strategies(&cfg).len(), 5);
    }
}
