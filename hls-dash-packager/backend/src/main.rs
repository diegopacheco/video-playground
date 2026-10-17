use anyhow::{Context, bail};
use packager::ladder::Ladder;
use packager::pipeline::load_manifest;
use packager::server::{AppState, router};
use std::path::PathBuf;
use std::sync::Arc;

fn usage() -> anyhow::Result<()> {
    bail!("usage: packager package <source.mp4> <work_dir> <out_dir> | packager serve <out_dir> <port>")
}

fn package(source: &str, work: &str, out: &str) -> anyhow::Result<()> {
    let manifest = packager::ffmpeg_pipeline(Ladder::standard())
        .run(&PathBuf::from(source), &PathBuf::from(work), &PathBuf::from(out))
        .context("packaging failed")?;
    for r in &manifest.renditions {
        println!(
            "{:>6} {}x{} target {}k avg {:.0}k peak {:.0}k segments {} bytes {}",
            r.id,
            r.width,
            r.height,
            r.target_kbps,
            r.average_kbps,
            r.peak_kbps,
            r.segments.len(),
            r.total_bytes
        );
    }
    println!(
        "keyframes aligned: {} segment boundaries on keyframes: {}",
        manifest.alignment.aligned, manifest.alignment.boundaries_on_keyframes
    );
    if !manifest.alignment.ok() {
        bail!("alignment failed: {:?}", manifest.alignment.issues);
    }
    Ok(())
}

async fn serve(out: &str, port: &str) -> anyhow::Result<()> {
    let out_dir = PathBuf::from(out);
    let manifest = load_manifest(&out_dir).context("no manifest.json, run `packager package` first")?;
    let app = router(AppState {
        out_dir,
        manifest: Arc::new(manifest),
    });
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("listening on http://localhost:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["package", source, work, out] => package(source, work, out),
        ["serve", out, port] => serve(out, port).await,
        _ => usage(),
    }
}
