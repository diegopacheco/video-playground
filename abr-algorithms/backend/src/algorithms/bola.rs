use super::{AbrAlgorithm, Decision};

pub const ID: &str = "bola";

pub struct Bola {
    pub min_buffer_s: f64,
}

impl Default for Bola {
    fn default() -> Self {
        Bola { min_buffer_s: 10.0 }
    }
}

pub struct BolaParams {
    pub utilities: Vec<f64>,
    pub gp: f64,
    pub vp: f64,
}

impl Bola {
    pub fn target_buffer_s(&self, decision: &Decision) -> f64 {
        (decision.max_buffer_s - decision.segment_s).max(self.min_buffer_s * 2.0)
    }

    pub fn params(&self, decision: &Decision) -> BolaParams {
        let lowest = decision.ladder_kbps[0];
        let utilities: Vec<f64> = decision.ladder_kbps.iter().map(|b| (b / lowest).ln() + 1.0).collect();
        let top = utilities[utilities.len() - 1];
        let gp = (top - 1.0) / (self.target_buffer_s(decision) / self.min_buffer_s - 1.0);
        let vp = self.min_buffer_s / gp;
        BolaParams { utilities, gp, vp }
    }

    pub fn scores(&self, decision: &Decision) -> Vec<f64> {
        let p = self.params(decision);
        decision
            .ladder_kbps
            .iter()
            .zip(&p.utilities)
            .map(|(bitrate, utility)| (p.vp * (utility + p.gp) - decision.buffer_s) / (bitrate * decision.segment_s))
            .collect()
    }
}

impl AbrAlgorithm for Bola {
    fn id(&self) -> &'static str {
        ID
    }

    fn choose(&mut self, decision: &Decision) -> usize {
        self.scores(decision)
            .iter()
            .enumerate()
            .fold((0, f64::NEG_INFINITY), |best, (index, score)| if *score > best.1 { (index, *score) } else { best })
            .0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LADDER: [f64; 6] = [300.0, 750.0, 1200.0, 1850.0, 2850.0, 4300.0];
    const FAST: [f64; 5] = [50_000.0; 5];

    fn decide(buffer_s: f64, history: &[f64]) -> usize {
        Bola::default().choose(&Decision {
            ladder_kbps: &LADDER,
            segment_s: 4.0,
            max_buffer_s: 30.0,
            buffer_s,
            throughput_kbps: history,
            last_quality: Some(5),
        })
    }

    #[test]
    fn never_picks_above_lowest_while_buffer_is_below_minimum_even_on_a_fast_network() {
        let mut buffer = 0.0;
        while buffer < 10.0 {
            assert_eq!(decide(buffer, &FAST), 0, "buffer {buffer}s is the danger zone, BOLA must protect it");
            buffer += 0.25;
        }
    }

    #[test]
    fn quality_never_decreases_as_buffer_grows() {
        let mut previous = 0;
        let mut buffer = 0.0;
        while buffer <= 30.0 {
            let quality = decide(buffer, &[]);
            assert!(quality >= previous, "at {buffer}s quality dropped from {previous} to {quality}");
            previous = quality;
            buffer += 0.1;
        }
    }

    #[test]
    fn reaches_top_quality_before_the_buffer_is_full() {
        assert_eq!(decide(26.0, &[]), 5, "26s is the highest buffer seen at decision time with a 30s cap and 4s segments");
    }

    #[test]
    fn decision_depends_only_on_buffer_not_on_throughput() {
        for buffer in [0.0, 12.0, 15.0, 20.0, 24.0] {
            assert_eq!(decide(buffer, &[100.0; 5]), decide(buffer, &FAST));
        }
    }

    #[test]
    fn utility_is_log_of_bitrate_ratio_shifted_to_one() {
        let bola = Bola::default();
        let decision = Decision {
            ladder_kbps: &LADDER,
            segment_s: 4.0,
            max_buffer_s: 30.0,
            buffer_s: 0.0,
            throughput_kbps: &[],
            last_quality: None,
        };
        let p = bola.params(&decision);
        assert!((p.utilities[0] - 1.0).abs() < 1e-12);
        assert!((p.utilities[5] - (1.0 + (4300.0f64 / 300.0).ln())).abs() < 1e-12);
        assert!((p.vp * p.gp - 10.0).abs() < 1e-9, "Vp * gp equals the minimum buffer by construction");
    }
}
