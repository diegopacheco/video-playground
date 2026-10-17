use std::sync::Arc;
use streamlimit::clock::SystemClock;
use streamlimit::http::router;
use streamlimit::plan::{Policy, default_accounts};
use streamlimit::redis_store::RedisLeaseStore;
use streamlimit::service::LeaseService;

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = env_or("PORT", "8111");
    let redis_url = env_or("REDIS_URL", "redis://127.0.0.1:6111");
    let ttl_ms: u64 = env_or("LEASE_TTL_MS", "10000").parse()?;
    let policy = Policy::parse(&env_or("POLICY", "reject_newest"))?;
    let store = RedisLeaseStore::connect(&redis_url, "sl").await?;
    let svc = Arc::new(LeaseService::new(
        store,
        Arc::new(SystemClock),
        ttl_ms,
        default_accounts(),
        policy,
    ));
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    println!("streamlimit listening on http://127.0.0.1:{port} redis {redis_url} ttl {ttl_ms}ms");
    axum::serve(listener, router(svc)).await?;
    Ok(())
}
