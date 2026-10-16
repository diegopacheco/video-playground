use crate::AbrError;
use crate::algorithms::{AbrAlgorithm, Decision};
use crate::trace::Trace;
use serde::Serialize;

const STALL_EPSILON_S: f64 = 1e-9;

#[derive(Debug, Clone, Serialize)]
pub struct PlayerConfig {
    pub ladder_kbps: Vec<f64>,
    pub segment_s: f64,
    pub segments: usize,
    pub max_buffer_s: f64,
    pub startup_buffer_s: f64,
    pub rtt_s: f64,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        PlayerConfig {
            ladder_kbps: vec![300.0, 750.0, 1200.0, 1850.0, 2850.0, 4300.0],
            segment_s: 4.0,
            segments: 150,
            max_buffer_s: 30.0,
            startup_buffer_s: 4.0,
            rtt_s: 0.08,
        }
    }
}

impl PlayerConfig {
    pub fn validate(&self) -> Result<(), AbrError> {
        let invalid = |reason: &str| Err(AbrError::InvalidConfig(reason.to_string()));
        if self.ladder_kbps.is_empty() || self.ladder_kbps.iter().any(|b| *b <= 0.0) {
            return invalid("ladder needs positive bitrates");
        }
        if self.ladder_kbps.windows(2).any(|w| w[0] >= w[1]) {
            return invalid("ladder must be strictly ascending");
        }
        if self.segment_s <= 0.0 || self.segments == 0 {
            return invalid("segments need a positive duration and count");
        }
        if self.max_buffer_s < self.segment_s {
            return invalid("max buffer must hold at least one segment");
        }
        if self.startup_buffer_s < 0.0 || self.rtt_s < 0.0 {
            return invalid("startup buffer and rtt cannot be negative");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SegmentRecord {
    pub index: usize,
    pub quality: usize,
    pub bitrate_kbps: f64,
    pub request_s: f64,
    pub buffer_start_s: f64,
    pub done_s: f64,
    pub throughput_kbps: f64,
    pub buffer_s: f64,
    pub stall_s: f64,
    pub idle_s: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub segments: Vec<SegmentRecord>,
    pub startup_delay_s: f64,
    pub rebuffer_s: f64,
    pub rebuffer_events: usize,
    pub content_s: f64,
    pub end_s: f64,
}

pub fn simulate(config: &PlayerConfig, trace: &Trace, algorithm: &mut dyn AbrAlgorithm) -> Result<Session, AbrError> {
    config.validate()?;
    let mut now = 0.0;
    let mut buffer = 0.0;
    let mut startup: Option<f64> = None;
    let mut rebuffer = 0.0;
    let mut events = 0;
    let mut throughput = Vec::with_capacity(config.segments);
    let mut segments = Vec::with_capacity(config.segments);
    let mut last_quality = None;

    for index in 0..config.segments {
        let playing = startup.is_some();
        let idle = if playing { (buffer + config.segment_s - config.max_buffer_s).max(0.0) } else { 0.0 };
        now += idle;
        buffer -= idle;
        let buffer_start_s = buffer;

        let quality = algorithm.choose(&Decision {
            ladder_kbps: &config.ladder_kbps,
            segment_s: config.segment_s,
            max_buffer_s: config.max_buffer_s,
            buffer_s: buffer,
            throughput_kbps: &throughput,
            last_quality,
        });
        let bitrate = *config.ladder_kbps.get(quality).ok_or_else(|| AbrError::InvalidQuality {
            algorithm: algorithm.id().to_string(),
            quality,
        })?;

        let kbits = bitrate * config.segment_s;
        let download = config.rtt_s + trace.download_time(now + config.rtt_s, kbits);
        let mut stall = 0.0;
        if playing {
            stall = (download - buffer).max(0.0);
            buffer = (buffer - download).max(0.0);
            if stall > STALL_EPSILON_S {
                rebuffer += stall;
                events += 1;
            } else {
                stall = 0.0;
            }
        }
        let request = now;
        now += download;
        buffer += config.segment_s;
        if startup.is_none() && buffer >= config.startup_buffer_s {
            startup = Some(now);
        }

        let measured = kbits / download;
        throughput.push(measured);
        last_quality = Some(quality);
        segments.push(SegmentRecord {
            index,
            quality,
            bitrate_kbps: bitrate,
            request_s: request,
            buffer_start_s,
            done_s: now,
            throughput_kbps: measured,
            buffer_s: buffer,
            stall_s: stall,
            idle_s: idle,
        });
    }

    Ok(Session {
        segments,
        startup_delay_s: startup.unwrap_or(now),
        rebuffer_s: rebuffer,
        rebuffer_events: events,
        content_s: config.segments as f64 * config.segment_s,
        end_s: now + buffer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed(usize);

    impl AbrAlgorithm for Fixed {
        fn id(&self) -> &'static str {
            "fixed"
        }
        fn choose(&mut self, _: &Decision) -> usize {
            self.0
        }
    }

    fn config(segments: usize) -> PlayerConfig {
        PlayerConfig { segments, rtt_s: 0.0, ..PlayerConfig::default() }
    }

    #[test]
    fn network_at_half_the_bitrate_stalls_exactly_one_segment_per_segment_after_startup() {
        let trace = Trace::new("half", vec![2150.0]).unwrap();
        let session = simulate(&config(10), &trace, &mut Fixed(5)).unwrap();
        assert!((session.startup_delay_s - 8.0).abs() < 1e-9, "first 4s segment at half speed takes 8s");
        assert!((session.rebuffer_s - 36.0).abs() < 1e-9, "9 later segments each starve 4s");
        assert_eq!(session.rebuffer_events, 9);
    }

    #[test]
    fn ample_bandwidth_never_rebuffers_and_buffer_respects_the_cap() {
        let trace = Trace::new("fast", vec![20_000.0]).unwrap();
        let session = simulate(&PlayerConfig::default(), &trace, &mut Fixed(5)).unwrap();
        assert_eq!(session.rebuffer_events, 0);
        assert_eq!(session.rebuffer_s, 0.0);
        assert!(session.segments.iter().all(|s| s.buffer_s <= 30.0 + 1e-9));
        assert!(session.segments.iter().any(|s| s.idle_s > 0.0), "player must pause downloads when the buffer is full");
    }

    #[test]
    fn startup_time_is_not_counted_as_rebuffering() {
        let trace = Trace::new("slow", vec![150.0]).unwrap();
        let session = simulate(&config(1), &trace, &mut Fixed(0)).unwrap();
        assert!((session.startup_delay_s - 8.0).abs() < 1e-9);
        assert_eq!(session.rebuffer_s, 0.0, "waiting for the first frame is startup delay, not a stall");
    }

    #[test]
    fn outage_mid_session_is_charged_as_one_stall() {
        let mut bandwidth = vec![10_000.0; 20];
        bandwidth.extend(vec![0.0; 40]);
        bandwidth.extend(vec![10_000.0; 100]);
        let trace = Trace::new("outage", bandwidth).unwrap();
        let cfg = PlayerConfig { segments: 30, max_buffer_s: 12.0, rtt_s: 0.0, ..PlayerConfig::default() };
        let session = simulate(&cfg, &trace, &mut Fixed(0)).unwrap();
        assert_eq!(session.rebuffer_events, 1);
        assert!(session.rebuffer_s > 25.0 && session.rebuffer_s < 40.0, "40s outage minus at most a 12s buffer, got {}", session.rebuffer_s);
    }

    #[test]
    fn rejects_algorithm_choosing_outside_the_ladder() {
        let trace = Trace::new("t", vec![1000.0]).unwrap();
        let err = simulate(&config(2), &trace, &mut Fixed(9)).unwrap_err();
        assert_eq!(err, AbrError::InvalidQuality { algorithm: "fixed".into(), quality: 9 });
    }

    #[test]
    fn rejects_descending_ladder() {
        let cfg = PlayerConfig { ladder_kbps: vec![1000.0, 500.0], ..PlayerConfig::default() };
        assert!(matches!(cfg.validate(), Err(AbrError::InvalidConfig(_))));
    }
}
