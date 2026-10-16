use simulcast_scheduler::api;
use simulcast_scheduler::clock::{Clock, OffsetClock, SystemClock};
use simulcast_scheduler::geo::HeaderThenIpResolver;
use simulcast_scheduler::herd::jitter::Jitter;
use simulcast_scheduler::seed;
use simulcast_scheduler::service::{HerdConfig, Scheduler};
use simulcast_scheduler::store::InMemoryStore;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8107".into()).parse()?;
    let clock = Arc::new(OffsetClock::new(Arc::new(SystemClock)));
    let scheduler = Arc::new(Scheduler::new(
        Arc::new(InMemoryStore::default()),
        Arc::new(HeaderThenIpResolver::with_default_table()),
        clock.clone(),
        Jitter::from_time(),
        HerdConfig::default(),
    ));
    for episode in seed::episodes(clock.now()) {
        scheduler.upsert(episode)?;
    }

    let warmer = scheduler.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_millis(200));
        loop {
            tick.tick().await;
            if let Err(e) = warmer.warm() {
                eprintln!("warm failed: {e}");
            }
        }
    });

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("simulcast scheduler listening on http://localhost:{port}");
    axum::serve(
        listener,
        api::router(scheduler).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
