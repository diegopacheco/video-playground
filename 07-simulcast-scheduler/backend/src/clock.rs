use chrono::{DateTime, Duration, Utc};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

pub struct ManualClock {
    now: Mutex<DateTime<Utc>>,
}

impl ManualClock {
    pub fn new(start: DateTime<Utc>) -> Self {
        Self { now: Mutex::new(start) }
    }

    pub fn set(&self, instant: DateTime<Utc>) {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner) = instant;
    }

    pub fn advance(&self, by: Duration) {
        let mut now = self.now.lock().unwrap_or_else(PoisonError::into_inner);
        *now += by;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub struct OffsetClock {
    base: Arc<dyn Clock>,
    offset_ms: AtomicI64,
}

impl OffsetClock {
    pub fn new(base: Arc<dyn Clock>) -> Self {
        Self { base, offset_ms: AtomicI64::new(0) }
    }

    pub fn set_offset_ms(&self, offset_ms: i64) {
        self.offset_ms.store(offset_ms, Ordering::Relaxed);
    }

    pub fn offset_ms(&self) -> i64 {
        self.offset_ms.load(Ordering::Relaxed)
    }
}

impl Clock for OffsetClock {
    fn now(&self) -> DateTime<Utc> {
        self.base.now() + Duration::milliseconds(self.offset_ms())
    }
}
