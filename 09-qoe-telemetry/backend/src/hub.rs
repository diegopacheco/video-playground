use std::collections::VecDeque;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tokio::sync::{broadcast, mpsc};

use crate::aggregator::{Aggregator, WindowResult};
use crate::beacon::Beacon;

#[derive(Debug, Clone, Serialize)]
pub struct WindowBatch {
    pub results: Vec<WindowResult>,
    pub consumed: u64,
    pub late_dropped: u64,
}

pub struct Hub {
    tx: broadcast::Sender<WindowBatch>,
    history: RwLock<VecDeque<WindowResult>>,
    capacity: usize,
}

impl Hub {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(64);
        Self {
            tx,
            history: RwLock::new(VecDeque::new()),
            capacity,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WindowBatch> {
        self.tx.subscribe()
    }

    pub fn history(&self) -> Vec<WindowResult> {
        self.history
            .read()
            .map(|h| h.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn publish(&self, batch: WindowBatch) {
        if let Ok(mut history) = self.history.write() {
            history.extend(batch.results.iter().cloned());
            while history.len() > self.capacity {
                history.pop_front();
            }
        }
        let _ = self.tx.send(batch);
    }
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

pub async fn run_windows(
    mut rx: mpsc::Receiver<Beacon>,
    mut aggregator: Box<dyn Aggregator>,
    hub: std::sync::Arc<Hub>,
    lateness_ms: i64,
) {
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    let mut consumed = 0u64;
    loop {
        tokio::select! {
            next = rx.recv() => match next {
                Some(beacon) => {
                    consumed += 1;
                    aggregator.ingest(&beacon);
                }
                None => return,
            },
            _ = tick.tick() => {
                let results = aggregator.advance(now_ms() - lateness_ms);
                hub.publish(WindowBatch {
                    results,
                    consumed,
                    late_dropped: aggregator.late_dropped(),
                });
            }
        }
    }
}
