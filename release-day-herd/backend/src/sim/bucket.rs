const SCALE: u64 = 1_000_000;

#[derive(Debug, Clone)]
pub struct TokenBucket {
    rate_per_s: u64,
    capacity: u64,
    level: u64,
    last_us: u64,
}

impl TokenBucket {
    pub fn new(rate_per_s: u64, burst: u64) -> Self {
        let capacity = burst.max(1) * SCALE;
        Self { rate_per_s, capacity, level: capacity, last_us: 0 }
    }

    pub fn for_rate(rate_per_s: u64) -> Self {
        Self::new(rate_per_s, rate_per_s / 20)
    }

    pub fn burst(&self) -> u64 {
        self.capacity / SCALE
    }

    pub fn try_take(&mut self, now_us: u64) -> bool {
        if now_us > self.last_us {
            let refill = (now_us - self.last_us).saturating_mul(self.rate_per_s);
            self.level = self.level.saturating_add(refill).min(self.capacity);
            self.last_us = now_us;
        }
        if self.level >= SCALE {
            self.level -= SCALE;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_holds_even_when_hammered_every_microsecond() {
        let mut bucket = TokenBucket::for_rate(1_000);
        let mut per_second = [0u64; 5];
        for t in 0..5_000_000u64 {
            if bucket.try_take(t) {
                per_second[(t / SCALE) as usize] += 1;
            }
        }
        assert!(per_second.iter().all(|&n| n <= 1_000 + bucket.burst()));
        assert!(per_second[1..].iter().all(|&n| n >= 990));
    }

    #[test]
    fn idle_time_only_banks_up_to_the_burst() {
        let mut bucket = TokenBucket::new(100, 5);
        let granted = (0..50).filter(|_| bucket.try_take(60_000_000)).count();
        assert_eq!(granted, 5);
    }
}
