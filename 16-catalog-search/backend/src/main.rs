use std::sync::Arc;

use catalog_search::api::router;
use catalog_search::catalog::default_catalog;
use catalog_search::engine::DefaultEngine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8116".into()).parse()?;
    let engine = Arc::new(DefaultEngine::with_defaults(default_catalog()?));
    let stats = engine.stats();
    println!(
        "indexed {} titles, {} grams, {} postings",
        stats.documents, stats.grams, stats.postings
    );
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("catalog-search listening on http://127.0.0.1:{port}");
    axum::serve(listener, router(engine)).await?;
    Ok(())
}
