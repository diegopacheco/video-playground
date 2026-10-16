use std::sync::{Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Jitter {
    state: Mutex<u64>,
}

impl Jitter {
    pub fn seeded(seed: u64) -> Self {
        Self { state: Mutex::new(seed.max(1)) }
    }

    pub fn from_time() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Self::seeded(nanos)
    }

    fn next(&self) -> u64 {
        let mut x = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        *x ^= *x << 13;
        *x ^= *x >> 7;
        *x ^= *x << 17;
        *x
    }

    pub fn below(&self, bound: u64) -> u64 {
        if bound == 0 { 0 } else { self.next() % bound }
    }

    pub fn spread(&self, base: u64, pct: u64) -> u64 {
        let width = base * pct / 100;
        base - width + self.below(2 * width + 1)
    }
}
