use crate::error::LeaseError;
use crate::plan::Policy;
use serde::Serialize;
use std::future::Future;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Lease {
    pub device: String,
    pub started_at_ms: u64,
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone)]
pub struct AcquireRequest<'a> {
    pub account: &'a str,
    pub device: &'a str,
    pub now_ms: u64,
    pub ttl_ms: u64,
    pub limit: usize,
    pub policy: Policy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcquireOutcome {
    Granted { lease: Lease, evicted: Vec<String> },
    Rejected { active: usize },
}

pub trait LeaseStore: Send + Sync + 'static {
    fn acquire(
        &self,
        req: AcquireRequest<'_>,
    ) -> impl Future<Output = Result<AcquireOutcome, LeaseError>> + Send;

    fn heartbeat(
        &self,
        account: &str,
        device: &str,
        now_ms: u64,
        ttl_ms: u64,
    ) -> impl Future<Output = Result<Option<Lease>, LeaseError>> + Send;

    fn release(
        &self,
        account: &str,
        device: &str,
    ) -> impl Future<Output = Result<bool, LeaseError>> + Send;

    fn list(
        &self,
        account: &str,
        now_ms: u64,
    ) -> impl Future<Output = Result<Vec<Lease>, LeaseError>> + Send;
}
