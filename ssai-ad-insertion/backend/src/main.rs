use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8105".into());
    let media_dir = PathBuf::from(std::env::var("MEDIA_DIR").unwrap_or_else(|_| "../media".into()));
    let state = ssai::build_state(&media_dir).context("loading media, run scripts/setup.sh first")?;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("ssai stitcher listening on http://localhost:{port}");
    axum::serve(listener, ssai::http::router(Arc::new(state))).await?;
    Ok(())
}
