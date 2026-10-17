use std::sync::Arc;

use anyhow::Context;
use subtitle_service::ass::AssParser;
use subtitle_service::catalog::{Catalog, FsTrackSource};
use subtitle_service::http::{AppState, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subs_dir = std::env::var("SUBS_DIR").unwrap_or_else(|_| "../subs".into());
    let media_dir = std::env::var("MEDIA_DIR").unwrap_or_else(|_| "../media".into());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8106".into())
        .parse()
        .context("PORT must be a number")?;
    let catalog = Catalog::build(&FsTrackSource::new(&subs_dir), &AssParser)
        .with_context(|| format!("loading subtitles from {subs_dir}"))?;
    for track in catalog.tracks() {
        println!(
            "{} ass={}B vtt={}B fidelity={} convert={}us",
            track.locale, track.ass_bytes, track.vtt_bytes, track.report.fidelity, track.convert_micros
        );
    }
    let state = AppState {
        catalog: Arc::new(catalog),
        media_dir: media_dir.into(),
    };
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .with_context(|| format!("binding port {port}"))?;
    println!("listening on http://localhost:{port}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
