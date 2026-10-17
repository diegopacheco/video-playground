use std::sync::Arc;
use std::time::Duration;

use resume_watching::api::{AppState, router};
use resume_watching::coalescer::Coalescer;
use resume_watching::config::Config;
use resume_watching::store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let store: Arc<dyn store::PlayheadStore> = store::connect(&config.store, &config.redis_url, &config.scylla_node).await?.into();
    let coalescer = Arc::new(Coalescer::new());

    let flusher = (coalescer.clone(), store.clone());
    let every = Duration::from_secs(config.flush_secs);
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(every);
        loop {
            tick.tick().await;
            if let Err(e) = flusher.0.flush(flusher.1.as_ref()).await {
                eprintln!("flush failed, positions requeued: {e}");
            }
        }
    });

    let state = AppState { store: store.clone(), coalescer, flush_secs: config.flush_secs };
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", config.port)).await?;
    println!("resume-watching on port {} using {} flush every {}s", config.port, store.name(), config.flush_secs);
    axum::serve(listener, router(state)).await?;
    Ok(())
}
