use anyhow::{Context, bail};
use llhls::ingest::ingest;
use llhls::live::{LiveStream, WindowConfig};
use llhls::playlist::{LowLatencyPlaylist, RegularPlaylist};
use llhls::server::{AppState, router};
use llhls::source::{FfmpegSource, LiveSource};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8104".into()).parse().context("PORT")?;
    let source = FfmpegSource { fps: 30, segment_seconds: 2, part_ms: 200 };
    let config = WindowConfig {
        max_segments: 12,
        part_target: source.part_ms as f64 / 1000.0,
        segment_target: source.segment_seconds as f64,
    };
    let live = Arc::new(LiveStream::new(config));
    let running = source.start()?;
    let ingest_live = live.clone();
    let output = running.output;
    let epoch_ms = running.epoch_ms;
    let ingest_task = tokio::spawn(async move { ingest(output, &ingest_live, epoch_ms).await });
    let state = Arc::new(AppState {
        live,
        low_latency: Box::new(LowLatencyPlaylist { part_hold_back_factor: 3.0, part_window_targets: 3.0 }),
        regular: Box::new(RegularPlaylist { hold_back_segments: 3.0 }),
        block_timeout: Duration::from_secs_f64(config.segment_target * 3.0),
    });
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await.context("bind")?;
    println!("origin listening on http://localhost:{port}/live/ll.m3u8 and /live/hls.m3u8");
    tokio::select! {
        served = axum::serve(listener, router(state)) => served.context("server")?,
        ingested = ingest_task => bail!("ingest stopped: {:?}", ingested?),
        fed = running.task => bail!("source stopped: {:?}", fed?),
    }
    Ok(())
}
