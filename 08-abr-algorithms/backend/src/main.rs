use abr::api::{AppState, router};
use abr::player::PlayerConfig;
use abr::trace::bundled;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8108".into()).parse()?;
    let state = Arc::new(AppState {
        traces: bundled()?,
        config: PlayerConfig::default(),
    });
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("abr simulator listening on http://localhost:{port}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
