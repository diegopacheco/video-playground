use super::{AbrAlgorithm, Bola, Decision, Throughput};

pub const ID: &str = "dynamic";

pub struct Dynamic {
    pub throughput: Throughput,
    pub bola: Bola,
    pub switch_on_s: f64,
    pub switch_off_s: f64,
    pub using_bola: bool,
}

impl Default for Dynamic {
    fn default() -> Self {
        Dynamic {
            throughput: Throughput::default(),
            bola: Bola::default(),
            switch_on_s: 10.0,
            switch_off_s: 5.0,
            using_bola: false,
        }
    }
}

impl AbrAlgorithm for Dynamic {
    fn id(&self) -> &'static str {
        ID
    }

    fn choose(&mut self, decision: &Decision) -> usize {
        self.using_bola = if self.using_bola {
            decision.buffer_s >= self.switch_off_s
        } else {
            decision.buffer_s >= self.switch_on_s
        };
        if self.using_bola {
            self.bola.choose(decision)
        } else {
            self.throughput.choose(decision)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LADDER: [f64; 6] = [300.0, 750.0, 1200.0, 1850.0, 2850.0, 4300.0];

    fn decision<'a>(buffer_s: f64, history: &'a [f64]) -> Decision<'a> {
        Decision {
            ladder_kbps: &LADDER,
            segment_s: 4.0,
            max_buffer_s: 30.0,
            buffer_s,
            throughput_kbps: history,
            last_quality: None,
        }
    }

    #[test]
    fn startup_uses_throughput_so_a_fast_network_is_not_wasted_on_lowest_quality() {
        let mut dynamic = Dynamic::default();
        let fast = [6000.0; 3];
        assert_eq!(dynamic.choose(&decision(4.0, &fast)), 5);
        assert_eq!(Bola::default().choose(&decision(4.0, &fast)), 0);
    }

    #[test]
    fn steady_state_follows_bola_and_ignores_a_throughput_dip() {
        let mut dynamic = Dynamic::default();
        let slow = [400.0; 5];
        let d = decision(24.0, &slow);
        assert_eq!(dynamic.choose(&d), Bola::default().choose(&d));
        assert!(dynamic.using_bola);
    }

    #[test]
    fn hysteresis_keeps_bola_between_off_and_on_thresholds() {
        let mut dynamic = Dynamic::default();
        dynamic.choose(&decision(12.0, &[]));
        assert!(dynamic.using_bola);
        dynamic.choose(&decision(7.0, &[]));
        assert!(dynamic.using_bola, "7s is above the 5s switch off threshold");
        dynamic.choose(&decision(4.0, &[]));
        assert!(!dynamic.using_bola, "draining below 5s hands control back to throughput");
        dynamic.choose(&decision(7.0, &[]));
        assert!(!dynamic.using_bola, "7s is below the 10s switch on threshold");
    }
}
