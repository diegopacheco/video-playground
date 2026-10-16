use crate::player::Session;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Metrics {
    pub avg_bitrate_kbps: f64,
    pub switch_count: usize,
    pub rebuffer_s: f64,
    pub rebuffer_events: usize,
    pub rebuffer_ratio: f64,
    pub startup_delay_s: f64,
    pub qoe: f64,
}

pub struct QoeWeights {
    pub switch_penalty: f64,
    pub rebuffer_penalty_per_s: f64,
    pub startup_penalty_per_s: f64,
}

impl QoeWeights {
    pub fn for_top_bitrate(top_kbps: f64) -> QoeWeights {
        let top_mbps = top_kbps / 1000.0;
        QoeWeights {
            switch_penalty: 1.0,
            rebuffer_penalty_per_s: top_mbps,
            startup_penalty_per_s: top_mbps,
        }
    }
}

pub fn evaluate(session: &Session, weights: &QoeWeights) -> Metrics {
    let bitrates: Vec<f64> = session.segments.iter().map(|s| s.bitrate_kbps).collect();
    let count = bitrates.len().max(1) as f64;
    let switch_count = session.segments.windows(2).filter(|w| w[0].quality != w[1].quality).count();
    let quality_mbps: f64 = bitrates.iter().sum::<f64>() / 1000.0;
    let variation_mbps: f64 = bitrates.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / 1000.0;
    let qoe = (quality_mbps
        - weights.switch_penalty * variation_mbps
        - weights.rebuffer_penalty_per_s * session.rebuffer_s
        - weights.startup_penalty_per_s * session.startup_delay_s)
        / count;
    Metrics {
        avg_bitrate_kbps: bitrates.iter().sum::<f64>() / count,
        switch_count,
        rebuffer_s: session.rebuffer_s,
        rebuffer_events: session.rebuffer_events,
        rebuffer_ratio: session.rebuffer_s / (session.content_s + session.rebuffer_s),
        startup_delay_s: session.startup_delay_s,
        qoe,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::SegmentRecord;

    fn session(qualities: &[(usize, f64)], rebuffer_s: f64, startup_s: f64) -> Session {
        Session {
            segments: qualities
                .iter()
                .enumerate()
                .map(|(index, (quality, bitrate))| SegmentRecord {
                    index,
                    quality: *quality,
                    bitrate_kbps: *bitrate,
                    request_s: 0.0,
                    buffer_start_s: 0.0,
                    done_s: 0.0,
                    throughput_kbps: 0.0,
                    buffer_s: 0.0,
                    stall_s: 0.0,
                    idle_s: 0.0,
                })
                .collect(),
            startup_delay_s: startup_s,
            rebuffer_s,
            rebuffer_events: usize::from(rebuffer_s > 0.0),
            content_s: qualities.len() as f64 * 4.0,
            end_s: 0.0,
        }
    }

    fn weights() -> QoeWeights {
        QoeWeights::for_top_bitrate(4300.0)
    }

    #[test]
    fn switch_count_counts_only_quality_changes() {
        let s = session(&[(0, 300.0), (0, 300.0), (2, 1200.0), (2, 1200.0), (0, 300.0)], 0.0, 0.0);
        assert_eq!(evaluate(&s, &weights()).switch_count, 2);
    }

    #[test]
    fn one_second_of_stall_costs_as_much_as_one_segment_at_top_quality() {
        let smooth = evaluate(&session(&[(5, 4300.0); 10], 0.0, 0.0), &weights());
        let stalled = evaluate(&session(&[(5, 4300.0); 10], 1.0, 0.0), &weights());
        assert!(((smooth.qoe - stalled.qoe) * 10.0 - 4.3).abs() < 1e-9);
    }

    #[test]
    fn oscillating_is_worse_than_steady_at_the_same_average() {
        let steady = evaluate(&session(&[(3, 1850.0); 4], 0.0, 0.0), &weights());
        let flapping = evaluate(&session(&[(2, 1200.0), (4, 2500.0), (2, 1200.0), (4, 2500.0)], 0.0, 0.0), &weights());
        assert!((steady.avg_bitrate_kbps - flapping.avg_bitrate_kbps).abs() < 1e-9);
        assert!(steady.qoe > flapping.qoe);
    }

    #[test]
    fn rebuffer_ratio_is_stall_share_of_watch_time() {
        let m = evaluate(&session(&[(0, 300.0); 10], 10.0, 0.0), &weights());
        assert!((m.rebuffer_ratio - 0.2).abs() < 1e-9, "10s stall over 40s content plus 10s stall");
    }
}
