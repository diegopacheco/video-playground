use std::collections::VecDeque;
use std::sync::Mutex;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub at: u64,
    pub kind: String,
    pub outcome: String,
    pub detail: String,
}

pub struct AuditLog {
    events: Mutex<VecDeque<AuditEvent>>,
    capacity: usize,
}

impl AuditLog {
    pub fn new(capacity: usize) -> Self {
        Self { events: Mutex::new(VecDeque::new()), capacity }
    }

    pub fn record(&self, at: u64, kind: &str, outcome: &str, detail: String) {
        if let Ok(mut events) = self.events.lock() {
            if events.len() == self.capacity {
                events.pop_front();
            }
            events.push_back(AuditEvent { at, kind: kind.into(), outcome: outcome.into(), detail });
        }
    }

    pub fn recent(&self) -> Vec<AuditEvent> {
        self.events.lock().map(|e| e.iter().rev().cloned().collect()).unwrap_or_default()
    }
}
