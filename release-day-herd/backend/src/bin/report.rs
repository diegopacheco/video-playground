use std::time::Instant;

use herd::http::compare_all;
use herd::sim::SimConfig;

fn ms(v: Option<u64>) -> String {
    v.map(|x| x.to_string()).unwrap_or_else(|| "-".into())
}

fn main() -> anyhow::Result<()> {
    let cfg = SimConfig::default();
    let started = Instant::now();
    let reports = compare_all(&cfg)?;
    println!(
        "users={} seed={} api_rps={} origin_rps={} admission_rps={} max_attempts={} wall_ms={}",
        cfg.users,
        cfg.seed,
        cfg.api_rps,
        cfg.origin_rps,
        cfg.admission_rps,
        cfg.max_attempts,
        started.elapsed().as_millis()
    );
    println!("| strategy | success | peak origin rps | peak api rps | p50 ttff ms | p95 ttff ms | p99 ttff ms | retries | wasted retries | origin reqs |");
    println!("|---|---|---|---|---|---|---|---|---|---|");
    for r in reports {
        println!(
            "| {} | {:.2}% | {} | {} | {} | {} | {} | {} | {} | {} |",
            r.strategy,
            r.success_rate * 100.0,
            r.peak_origin_rps,
            r.peak_api_rps,
            ms(r.ttff_p50_ms),
            ms(r.ttff_p95_ms),
            ms(r.ttff_p99_ms),
            r.retries,
            r.wasted_retries,
            r.origin_requests
        );
    }
    Ok(())
}
