use per_title::api::{AppState, router};
use per_title::ffmpeg_pipeline;
use per_title::store::{AnalysisStore, JsonFileStore};
use std::path::PathBuf;
use std::sync::Arc;

fn analyze(work_dir: PathBuf, store: &JsonFileStore) -> anyhow::Result<()> {
    let pipeline = ffmpeg_pipeline(work_dir);
    let progress = |done: usize, total: usize| {
        if done.is_multiple_of(10) || done == total {
            println!("progress {done}/{total}");
        }
    };
    let analysis = pipeline.run(&progress)?;
    store.save(&analysis)?;
    for t in &analysis.titles {
        println!(
            "{}: hull {} points, ladder {} rungs, equal-quality saving vs generic {:.1}%",
            t.name,
            t.hulls.get("vmaf").map(|h| h.len()).unwrap_or(0),
            t.ladder.len(),
            t.savings.saving_pct
        );
    }
    println!("analysis took {:.1}s", analysis.elapsed_secs);
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let work_dir = PathBuf::from(std::env::var("DATA_DIR").unwrap_or_else(|_| "data".into()));
    let store = JsonFileStore {
        path: work_dir.join("analysis.json"),
    };
    let command = std::env::args().nth(1).unwrap_or_else(|| "serve".into());
    match command.as_str() {
        "analyze" => analyze(work_dir, &store),
        "analyze-if-missing" => {
            if store.load()?.is_some() {
                println!("analysis cache present at {}", store.path.display());
                Ok(())
            } else {
                analyze(work_dir, &store)
            }
        }
        "serve" => {
            let port = std::env::var("PORT").unwrap_or_else(|_| "8102".into());
            let state = AppState::new(ffmpeg_pipeline(work_dir), Arc::new(store));
            let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
            println!("backend listening on http://localhost:{port}");
            axum::serve(listener, router(state)).await?;
            Ok(())
        }
        other => anyhow::bail!("unknown command {other}, use serve, analyze or analyze-if-missing"),
    }
}
