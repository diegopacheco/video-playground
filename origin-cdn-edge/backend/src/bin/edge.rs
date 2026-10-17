use cdn::cache::LruCache;
use cdn::config::env_or;
use cdn::edge::Edge;
use cdn::edge_server::router;
use cdn::origin_client::HttpOrigin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port: u16 = env_or("EDGE_PORT", 6103)?;
    let origin_url: String = env_or("ORIGIN_URL", "http://127.0.0.1:8103".to_string())?;
    let capacity: usize = env_or("EDGE_CACHE_BYTES", 8 * 1024 * 1024)?;
    let edge = Edge::new(LruCache::new(capacity), HttpOrigin::new(&origin_url));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("edge listening on http://127.0.0.1:{port} origin={origin_url} cache_bytes={capacity}");
    axum::serve(listener, router(edge)).await?;
    Ok(())
}
