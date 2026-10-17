use super::{AbrAlgorithm, Decision, highest_below};

pub const ID: &str = "throughput";

pub struct Throughput {
    pub window: usize,
    pub safety: f64,
}

impl Default for Throughput {
    fn default() -> Self {
        Throughput { window: 5, safety: 0.9 }
    }
}

pub fn harmonic_mean(samples: &[f64]) -> Option<f64> {
    if samples.is_empty() || samples.iter().any(|s| *s <= 0.0) {
        return None;
    }
    Some(samples.len() as f64 / samples.iter().map(|s| 1.0 / s).sum::<f64>())
}

impl Throughput {
    pub fn estimate(&self, history: &[f64]) -> Option<f64> {
        let start = history.len().saturating_sub(self.window);
        harmonic_mean(&history[start..]).map(|mean| mean * self.safety)
    }
}

impl AbrAlgorithm for Throughput {
    fn id(&self) -> &'static str {
        ID
    }

    fn choose(&mut self, decision: &Decision) -> usize {
        match self.estimate(decision.throughput_kbps) {
            Some(budget) => highest_below(decision.ladder_kbps, budget),
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LADDER: [f64; 6] = [300.0, 750.0, 1200.0, 1850.0, 2850.0, 4300.0];

    fn decide(history: &[f64]) -> usize {
        Throughput::default().choose(&Decision {
            ladder_kbps: &LADDER,
            segment_s: 4.0,
            max_buffer_s: 30.0,
            buffer_s: 10.0,
            throughput_kbps: history,
            last_quality: None,
        })
    }

    #[test]
    fn single_fast_spike_does_not_inflate_the_estimate() {
        let history = [1000.0, 1000.0, 1000.0, 1000.0, 100_000.0];
        assert_eq!(decide(&history), 1, "harmonic mean ~1250 * 0.9 fits 750 only, arithmetic mean would pick the top");
    }

    #[test]
    fn starts_at_lowest_quality_without_any_measurement() {
        assert_eq!(decide(&[]), 0);
    }

    #[test]
    fn only_recent_window_counts_so_it_reacts_to_improving_network() {
        let history = [100.0, 100.0, 100.0, 6000.0, 6000.0, 6000.0, 6000.0, 6000.0];
        assert_eq!(decide(&history), 5);
    }

    #[test]
    fn safety_margin_keeps_bitrate_below_measured_throughput() {
        assert_eq!(decide(&[1900.0; 5]), 2, "1900 * 0.9 = 1710 cannot afford 1850");
    }
}
