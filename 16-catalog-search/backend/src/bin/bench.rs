use catalog_search::catalog::default_catalog;
use catalog_search::engine::DefaultEngine;
use catalog_search::workload::measure;

fn main() -> anyhow::Result<()> {
    let engine = DefaultEngine::with_defaults(default_catalog()?);
    let stats = engine.stats();
    let latency = measure(&engine, 250)?;
    println!(
        "titles={} grams={} postings={}",
        stats.documents, stats.grams, stats.postings
    );
    println!(
        "samples={} p50={}us p95={}us p99={}us max={}us",
        latency.samples, latency.p50_us, latency.p95_us, latency.p99_us, latency.max_us
    );
    Ok(())
}
