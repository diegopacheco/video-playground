use crate::error::LeaseError;
use crate::lease::{AcquireOutcome, AcquireRequest, Lease, LeaseStore};
use crate::plan::Policy;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

type Accounts = HashMap<String, HashMap<String, Lease>>;

#[derive(Default)]
pub struct MemoryLeaseStore {
    accounts: Mutex<Accounts>,
}

impl MemoryLeaseStore {
    pub fn new() -> MemoryLeaseStore {
        MemoryLeaseStore::default()
    }

    fn lock(&self) -> Result<MutexGuard<'_, Accounts>, LeaseError> {
        self.accounts.lock().map_err(|_| LeaseError::Poisoned)
    }
}

fn prune(leases: &mut HashMap<String, Lease>, now_ms: u64) {
    leases.retain(|_, l| l.expires_at_ms > now_ms);
}

fn oldest(leases: &HashMap<String, Lease>) -> Option<String> {
    leases
        .values()
        .min_by(|a, b| (a.started_at_ms, &a.device).cmp(&(b.started_at_ms, &b.device)))
        .map(|l| l.device.clone())
}

fn sorted(leases: &HashMap<String, Lease>) -> Vec<Lease> {
    let mut out: Vec<Lease> = leases.values().cloned().collect();
    out.sort_by(|a, b| (a.started_at_ms, &a.device).cmp(&(b.started_at_ms, &b.device)));
    out
}

impl LeaseStore for MemoryLeaseStore {
    async fn acquire(&self, req: AcquireRequest<'_>) -> Result<AcquireOutcome, LeaseError> {
        let mut guard = self.lock()?;
        let leases = guard.entry(req.account.to_string()).or_default();
        prune(leases, req.now_ms);
        let expires_at_ms = req.now_ms + req.ttl_ms;
        if let Some(existing) = leases.get_mut(req.device) {
            existing.expires_at_ms = expires_at_ms;
            return Ok(AcquireOutcome::Granted {
                lease: existing.clone(),
                evicted: Vec::new(),
            });
        }
        let mut evicted = Vec::new();
        while leases.len() >= req.limit {
            if req.policy == Policy::RejectNewest {
                return Ok(AcquireOutcome::Rejected {
                    active: leases.len(),
                });
            }
            match oldest(leases) {
                Some(device) => {
                    leases.remove(&device);
                    evicted.push(device);
                }
                None => break,
            }
        }
        let lease = Lease {
            device: req.device.to_string(),
            started_at_ms: req.now_ms,
            expires_at_ms,
        };
        leases.insert(req.device.to_string(), lease.clone());
        Ok(AcquireOutcome::Granted { lease, evicted })
    }

    async fn heartbeat(
        &self,
        account: &str,
        device: &str,
        now_ms: u64,
        ttl_ms: u64,
    ) -> Result<Option<Lease>, LeaseError> {
        let mut guard = self.lock()?;
        let Some(leases) = guard.get_mut(account) else {
            return Ok(None);
        };
        prune(leases, now_ms);
        Ok(leases.get_mut(device).map(|l| {
            l.expires_at_ms = now_ms + ttl_ms;
            l.clone()
        }))
    }

    async fn release(&self, account: &str, device: &str) -> Result<bool, LeaseError> {
        let mut guard = self.lock()?;
        Ok(guard
            .get_mut(account)
            .map(|leases| leases.remove(device).is_some())
            .unwrap_or(false))
    }

    async fn list(&self, account: &str, now_ms: u64) -> Result<Vec<Lease>, LeaseError> {
        let mut guard = self.lock()?;
        let Some(leases) = guard.get_mut(account) else {
            return Ok(Vec::new());
        };
        prune(leases, now_ms);
        Ok(sorted(leases))
    }
}
