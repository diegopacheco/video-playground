use cdn::config::env_or;
use cdn::origin_server::{OriginConfig, OriginState, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port: u16 = env_or("ORIGIN_PORT", 8103)?;
    let config = OriginConfig {
        latency_ms: env_or("ORIGIN_LATENCY_MS", 80)?,
        segment_bytes: env_or("ORIGIN_SEGMENT_BYTES", 65536)?,
        max_age: env_or("ORIGIN_MAX_AGE", 10)?,
        stale_while_revalidate: env_or("ORIGIN_SWR", 30)?,
    };
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("origin listening on http://127.0.0.1:{port} {config:?}");
    axum::serve(listener, router(OriginState::new(config))).await?;
    Ok(())
}
