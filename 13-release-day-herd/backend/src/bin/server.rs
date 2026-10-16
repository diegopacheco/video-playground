use anyhow::Context;
use herd::http::{AppState, router};

fn env_u64(name: &str, fallback: u64) -> anyhow::Result<u64> {
    match std::env::var(name) {
        Ok(v) => v.parse().with_context(|| format!("{name} must be a number")),
        Err(_) => Ok(fallback),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = env_u64("PORT", 8113)?;
    let rate = env_u64("ROOM_RATE", 200)?;
    let state = AppState::new(rate)?;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port as u16)).await?;
    println!("herd server on http://localhost:{port} waiting room rate {rate}/s");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
