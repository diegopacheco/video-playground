use std::future::Future;
use std::sync::Mutex;

use crate::beacon::Beacon;
use crate::error::AppError;

pub trait BeaconSink: Send + Sync + 'static {
    fn publish(&self, beacons: &[Beacon]) -> impl Future<Output = Result<usize, AppError>> + Send;
}

#[derive(Default)]
pub struct MemorySink {
    stored: Mutex<Vec<Beacon>>,
}

impl MemorySink {
    pub fn stored(&self) -> Vec<Beacon> {
        self.stored.lock().map(|v| v.clone()).unwrap_or_default()
    }
}

impl BeaconSink for MemorySink {
    async fn publish(&self, beacons: &[Beacon]) -> Result<usize, AppError> {
        let mut guard = self
            .stored
            .lock()
            .map_err(|_| AppError::InvalidBeacon("sink poisoned".into()))?;
        guard.extend_from_slice(beacons);
        Ok(beacons.len())
    }
}
