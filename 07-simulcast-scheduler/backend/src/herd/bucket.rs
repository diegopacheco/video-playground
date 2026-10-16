use std::sync::{Mutex, PoisonError};

pub struct TokenBucket {
    capacity: f64,
    refill_per_sec: f64,
    state: Mutex<(f64, i64)>,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_per_sec: f64, now_ms: i64) -> Self {
        Self {
            capacity,
            refill_per_sec,
            state: Mutex::new((capacity, now_ms)),
        }
    }

    pub fn try_acquire(&self, now_ms: i64) -> bool {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let elapsed = (now_ms - state.1).max(0) as f64 / 1000.0;
        state.0 = (state.0 + elapsed * self.refill_per_sec).min(self.capacity);
        state.1 = state.1.max(now_ms);
        if state.0 >= 1.0 {
            state.0 -= 1.0;
            true
        } else {
            false
        }
    }

    pub fn tokens(&self) -> f64 {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).0
    }
}
