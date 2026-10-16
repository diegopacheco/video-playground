use std::sync::Arc;
use watch_party::clock::SystemClock;
use watch_party::hub::Hub;
use watch_party::room::InMemoryRooms;
use watch_party::ws::router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8114".into()).parse()?;
    let hub = Arc::new(Hub::new(Box::new(InMemoryRooms::new()), Arc::new(SystemClock)));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("watch-party listening on http://localhost:{port}");
    axum::serve(listener, router(hub)).await?;
    Ok(())
}
