use std::sync::Arc;

use qoe::aggregator::TumblingWindows;
use qoe::config::Config;
use qoe::http::{AppState, router};
use qoe::hub::{Hub, run_windows};
use qoe::kafka::{KafkaSink, consumer, ensure_topic, pump};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env();
    ensure_topic(&config.brokers, &config.topic, 3).await?;

    let hub = Arc::new(Hub::new(2_000));
    let (tx, rx) = mpsc::channel(10_000);
    let aggregator = Box::new(TumblingWindows::new(config.window_ms));
    tokio::spawn(run_windows(rx, aggregator, hub.clone(), config.lateness_ms));

    let stream_consumer = consumer(&config.brokers, &config.topic, &config.group)?;
    tokio::spawn(async move {
        if let Err(e) = pump(stream_consumer, tx).await {
            eprintln!("consumer stopped: {e}");
            std::process::exit(1);
        }
    });

    let sink = Arc::new(KafkaSink::new(&config.brokers, &config.topic)?);
    let app = router(AppState { sink, hub });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port)).await?;
    println!("qoe ingest listening on http://localhost:{}", config.port);
    axum::serve(listener, app).await?;
    Ok(())
}
