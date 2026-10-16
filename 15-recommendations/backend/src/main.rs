use recs::api::router;
use recs::service::Service;
use recs::store::pg::PgStore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://recs:recs@localhost:6115/recs".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8115".into());
    let store = PgStore::connect(&url).await?;
    let service = Arc::new(Service::bootstrap(store).await?);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("recs listening on http://localhost:{port}");
    axum::serve(listener, router(service)).await?;
    Ok(())
}
