use crate::HerdError;

const MICROS_PER_SECOND: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticket {
    pub index: u64,
    pub admit_at_us: u64,
    pub position: u64,
    pub eta_us: u64,
}

#[derive(Debug, Clone)]
pub struct AdmissionGate {
    rate_per_s: u64,
    interval_us: u64,
    next_free_us: u64,
    schedule: Vec<u64>,
}

impl AdmissionGate {
    pub fn new(rate_per_s: u64) -> Result<Self, HerdError> {
        if rate_per_s == 0 || rate_per_s > MICROS_PER_SECOND {
            return Err(HerdError::InvalidConfig(format!(
                "admission rate must be between 1 and {MICROS_PER_SECOND}, got {rate_per_s}"
            )));
        }
        Ok(Self {
            rate_per_s,
            interval_us: MICROS_PER_SECOND.div_ceil(rate_per_s),
            next_free_us: 0,
            schedule: Vec::new(),
        })
    }

    pub fn rate_per_s(&self) -> u64 {
        self.rate_per_s
    }

    pub fn joined(&self) -> u64 {
        self.schedule.len() as u64
    }

    pub fn join(&mut self, now_us: u64) -> Ticket {
        let admit_at_us = now_us.max(self.next_free_us);
        self.next_free_us = admit_at_us + self.interval_us;
        let index = self.joined();
        self.schedule.push(admit_at_us);
        Ticket {
            index,
            admit_at_us,
            position: self.position(index, now_us).unwrap_or(0),
            eta_us: admit_at_us - now_us,
        }
    }

    pub fn admitted_by(&self, now_us: u64) -> u64 {
        self.schedule.partition_point(|&t| t <= now_us) as u64
    }

    pub fn admit_at(&self, index: u64) -> Option<u64> {
        self.schedule.get(index as usize).copied()
    }

    pub fn position(&self, index: u64, now_us: u64) -> Option<u64> {
        let at = self.admit_at(index)?;
        if at <= now_us {
            Some(0)
        } else {
            Some(index + 1 - self.admitted_by(now_us))
        }
    }

    pub fn admitted_times(&self, now_us: u64) -> &[u64] {
        &self.schedule[..self.admitted_by(now_us) as usize]
    }
}

pub fn max_in_window(sorted_us: &[u64], window_us: u64) -> u64 {
    let mut best = 0;
    let mut start = 0;
    for end in 0..sorted_us.len() {
        while sorted_us[end] - sorted_us[start] >= window_us {
            start += 1;
        }
        best = best.max(end - start + 1);
    }
    best as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_burst_of_joiners_is_never_admitted_faster_than_the_rate() {
        let mut gate = AdmissionGate::new(300).unwrap();
        for _ in 0..5_000 {
            gate.join(0);
        }
        let all = gate.admitted_times(u64::MAX);
        assert_eq!(all.len(), 5_000);
        assert!(max_in_window(all, MICROS_PER_SECOND) <= 300);
    }

    #[test]
    fn eta_and_position_tell_the_viewer_the_truth() {
        let mut gate = AdmissionGate::new(10).unwrap();
        let first = gate.join(0);
        let tenth = (1..10).map(|_| gate.join(0)).last().unwrap();
        assert_eq!(first.eta_us, 0);
        assert_eq!(first.position, 0);
        assert_eq!(tenth.position, 9);
        assert_eq!(tenth.eta_us, 900_000);
        assert_eq!(gate.position(9, 899_999), Some(1));
        assert_eq!(gate.position(9, 900_000), Some(0));
    }

    #[test]
    fn an_idle_room_admits_a_late_joiner_immediately() {
        let mut gate = AdmissionGate::new(5).unwrap();
        gate.join(0);
        let late = gate.join(10_000_000);
        assert_eq!(late.eta_us, 0);
    }

    #[test]
    fn zero_rate_is_rejected() {
        assert!(AdmissionGate::new(0).is_err());
    }
}
