use crate::entry::Entry;
use crate::error::CdnError;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use tokio::sync::OnceCell;

type Flight = Arc<OnceCell<Result<Entry, CdnError>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Leader,
    Follower,
}

#[derive(Default)]
pub struct Coalescer {
    flights: Mutex<HashMap<String, Flight>>,
}

impl Coalescer {
    pub fn in_flight(&self) -> usize {
        self.flights.lock().map(|f| f.len()).unwrap_or(0)
    }

    pub async fn run<F, Fut>(&self, key: &str, fetch: F) -> (Result<Entry, CdnError>, Role)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Entry, CdnError>>,
    {
        let flight = match self.flights.lock() {
            Ok(mut flights) => flights.entry(key.to_string()).or_default().clone(),
            Err(_) => Flight::default(),
        };
        let mut led = false;
        let result = flight
            .get_or_init(|| {
                led = true;
                fetch()
            })
            .await
            .clone();
        if led
            && let Ok(mut flights) = self.flights.lock()
            && flights.get(key).is_some_and(|current| Arc::ptr_eq(current, &flight))
        {
            flights.remove(key);
        }
        let role = if led { Role::Leader } else { Role::Follower };
        (result, role)
    }
}
