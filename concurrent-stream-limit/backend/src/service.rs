use crate::clock::Clock;
use crate::error::LeaseError;
use crate::lease::{AcquireOutcome, AcquireRequest, Lease, LeaseStore};
use crate::plan::{Account, Policy};
use serde::Serialize;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub account: Account,
    pub policy: Policy,
    pub ttl_ms: u64,
    pub now_ms: u64,
    pub leases: Vec<Lease>,
}

pub struct LeaseService<S: LeaseStore> {
    store: S,
    clock: Arc<dyn Clock>,
    ttl_ms: u64,
    accounts: Vec<Account>,
    policy: RwLock<Policy>,
}

fn valid_device(device: &str) -> Result<(), LeaseError> {
    let ok = !device.is_empty()
        && device.len() <= 64
        && device
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok { Ok(()) } else { Err(LeaseError::InvalidDevice) }
}

impl<S: LeaseStore> LeaseService<S> {
    pub fn new(
        store: S,
        clock: Arc<dyn Clock>,
        ttl_ms: u64,
        accounts: Vec<Account>,
        policy: Policy,
    ) -> LeaseService<S> {
        LeaseService {
            store,
            clock,
            ttl_ms,
            accounts,
            policy: RwLock::new(policy),
        }
    }

    pub fn ttl_ms(&self) -> u64 {
        self.ttl_ms
    }

    pub fn accounts(&self) -> &[Account] {
        &self.accounts
    }

    pub fn account(&self, id: &str) -> Result<&Account, LeaseError> {
        self.accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| LeaseError::UnknownAccount(id.to_string()))
    }

    pub fn policy(&self) -> Result<Policy, LeaseError> {
        self.policy
            .read()
            .map(|p| *p)
            .map_err(|_| LeaseError::Poisoned)
    }

    pub fn set_policy(&self, policy: Policy) -> Result<(), LeaseError> {
        let mut guard = self.policy.write().map_err(|_| LeaseError::Poisoned)?;
        *guard = policy;
        Ok(())
    }

    pub async fn acquire(&self, account: &str, device: &str) -> Result<AcquireOutcome, LeaseError> {
        valid_device(device)?;
        let limit = self.account(account)?.limit;
        let policy = self.policy()?;
        let req = AcquireRequest {
            account,
            device,
            now_ms: self.clock.now_ms(),
            ttl_ms: self.ttl_ms,
            limit,
            policy,
        };
        self.store.acquire(req).await
    }

    pub async fn heartbeat(&self, account: &str, device: &str) -> Result<Option<Lease>, LeaseError> {
        valid_device(device)?;
        self.account(account)?;
        self.store
            .heartbeat(account, device, self.clock.now_ms(), self.ttl_ms)
            .await
    }

    pub async fn release(&self, account: &str, device: &str) -> Result<bool, LeaseError> {
        valid_device(device)?;
        self.account(account)?;
        self.store.release(account, device).await
    }

    pub async fn snapshot(&self, account: &str) -> Result<Snapshot, LeaseError> {
        let acc = self.account(account)?.clone();
        let now_ms = self.clock.now_ms();
        let leases = self.store.list(account, now_ms).await?;
        Ok(Snapshot {
            account: acc,
            policy: self.policy()?,
            ttl_ms: self.ttl_ms,
            now_ms,
            leases,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::ManualClock;
    use crate::memory::MemoryLeaseStore;
    use crate::plan::default_accounts;

    fn service(policy: Policy) -> (LeaseService<MemoryLeaseStore>, Arc<ManualClock>) {
        let clock = Arc::new(ManualClock::new(1_000));
        let svc = LeaseService::new(
            MemoryLeaseStore::new(),
            clock.clone(),
            10_000,
            default_accounts(),
            policy,
        );
        (svc, clock)
    }

    fn granted(o: &AcquireOutcome) -> bool {
        matches!(o, AcquireOutcome::Granted { .. })
    }

    #[tokio::test]
    async fn fan_plan_allows_two_screens_and_rejects_the_third() {
        let (svc, _) = service(Policy::RejectNewest);
        assert!(granted(&svc.acquire("bob", "tv").await.unwrap()));
        assert!(granted(&svc.acquire("bob", "phone").await.unwrap()));
        assert_eq!(
            svc.acquire("bob", "tablet").await.unwrap(),
            AcquireOutcome::Rejected { active: 2 }
        );
    }

    #[tokio::test]
    async fn the_same_device_reacquiring_does_not_burn_a_second_slot() {
        let (svc, _) = service(Policy::RejectNewest);
        assert!(granted(&svc.acquire("alice", "tv").await.unwrap()));
        assert!(granted(&svc.acquire("alice", "tv").await.unwrap()));
        assert_eq!(svc.snapshot("alice").await.unwrap().leases.len(), 1);
    }

    #[tokio::test]
    async fn crashed_client_slot_is_recovered_after_ttl_without_release() {
        let (svc, clock) = service(Policy::RejectNewest);
        assert!(granted(&svc.acquire("alice", "crashed-tv").await.unwrap()));
        clock.advance(9_999);
        assert!(!granted(&svc.acquire("alice", "phone").await.unwrap()));
        clock.advance(1);
        assert!(granted(&svc.acquire("alice", "phone").await.unwrap()));
    }

    #[tokio::test]
    async fn heartbeats_keep_a_long_session_alive_past_many_ttls() {
        let (svc, clock) = service(Policy::RejectNewest);
        svc.acquire("alice", "tv").await.unwrap();
        for _ in 0..10 {
            clock.advance(5_000);
            assert!(svc.heartbeat("alice", "tv").await.unwrap().is_some());
        }
        assert!(!granted(&svc.acquire("alice", "phone").await.unwrap()));
    }

    #[tokio::test]
    async fn heartbeat_after_expiry_is_refused_so_a_late_client_must_reacquire() {
        let (svc, clock) = service(Policy::RejectNewest);
        svc.acquire("alice", "tv").await.unwrap();
        clock.advance(10_000);
        assert!(svc.heartbeat("alice", "tv").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn kick_oldest_evicts_the_longest_running_stream_and_its_heartbeat_fails() {
        let (svc, clock) = service(Policy::KickOldest);
        svc.acquire("bob", "tv").await.unwrap();
        clock.advance(10);
        svc.acquire("bob", "phone").await.unwrap();
        clock.advance(10);
        svc.heartbeat("bob", "tv").await.unwrap();
        match svc.acquire("bob", "tablet").await.unwrap() {
            AcquireOutcome::Granted { evicted, .. } => assert_eq!(evicted, vec!["tv".to_string()]),
            other => panic!("expected grant, got {other:?}"),
        }
        assert!(svc.heartbeat("bob", "tv").await.unwrap().is_none());
        let devices: Vec<String> = svc
            .snapshot("bob")
            .await
            .unwrap()
            .leases
            .into_iter()
            .map(|l| l.device)
            .collect();
        assert_eq!(devices, vec!["phone".to_string(), "tablet".to_string()]);
    }

    #[tokio::test]
    async fn release_frees_the_slot_immediately() {
        let (svc, _) = service(Policy::RejectNewest);
        svc.acquire("alice", "tv").await.unwrap();
        assert!(svc.release("alice", "tv").await.unwrap());
        assert!(granted(&svc.acquire("alice", "phone").await.unwrap()));
    }

    #[tokio::test]
    async fn policy_switch_applies_to_the_next_acquire() {
        let (svc, _) = service(Policy::RejectNewest);
        svc.acquire("alice", "tv").await.unwrap();
        assert!(!granted(&svc.acquire("alice", "phone").await.unwrap()));
        svc.set_policy(Policy::KickOldest).unwrap();
        assert!(granted(&svc.acquire("alice", "phone").await.unwrap()));
    }

    #[tokio::test]
    async fn unknown_accounts_and_bad_device_ids_are_rejected() {
        let (svc, _) = service(Policy::RejectNewest);
        assert!(matches!(
            svc.acquire("mallory", "tv").await,
            Err(LeaseError::UnknownAccount(_))
        ));
        assert!(matches!(
            svc.acquire("alice", "tv:1 2").await,
            Err(LeaseError::InvalidDevice)
        ));
    }
}
