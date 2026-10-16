use std::time::Duration;

use qoe::hub::now_ms;
use qoe::synth::Fleet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let target = std::env::var("INGEST_URL")
        .unwrap_or_else(|_| "http://localhost:8109/api/beacons".to_string());
    let sessions: usize = std::env::var("SESSIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    let client = reqwest::Client::new();
    let mut fleet = Fleet::new(sessions, now_ms() as u64);
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    let mut sent = 0usize;
    println!("loadgen sending {sessions} concurrent sessions to {target}");
    loop {
        tick.tick().await;
        let batch = fleet.tick(now_ms());
        match client.post(&target).json(&batch).send().await {
            Ok(resp) if resp.status().is_success() => {
                sent += batch.len();
                println!("sent {} beacons, total {sent}", batch.len());
            }
            Ok(resp) => eprintln!("ingest rejected batch: {}", resp.status()),
            Err(e) => eprintln!("ingest unreachable: {e}"),
        }
    }
}
