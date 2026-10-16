use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, bail};
use resume_watching::coalescer::Coalescer;
use resume_watching::model::Playhead;
use resume_watching::store::{self, PlayheadStore};

struct Args {
    devices: u64,
    minutes: u64,
    heartbeat: u64,
    flush: u64,
    concurrency: usize,
    redis: String,
    scylla: String,
}

impl Args {
    fn parse() -> anyhow::Result<Self> {
        let mut a = Args {
            devices: 10000,
            minutes: 10,
            heartbeat: 10,
            flush: 60,
            concurrency: 64,
            redis: "redis://127.0.0.1:6110".into(),
            scylla: "127.0.0.1:6210".into(),
        };
        let raw: Vec<String> = std::env::args().skip(1).collect();
        for pair in raw.chunks(2) {
            let [key, value] = pair else { bail!("missing value for {}", pair[0]) };
            match key.as_str() {
                "--devices" => a.devices = value.parse()?,
                "--minutes" => a.minutes = value.parse()?,
                "--heartbeat" => a.heartbeat = value.parse()?,
                "--flush" => a.flush = value.parse()?,
                "--concurrency" => a.concurrency = value.parse()?,
                "--redis" => a.redis = value.clone(),
                "--scylla" => a.scylla = value.clone(),
                other => bail!("unknown flag {other}"),
            }
        }
        if a.devices < 2 || a.heartbeat == 0 || a.flush == 0 || a.concurrency == 0 {
            bail!("devices >= 2, heartbeat, flush and concurrency must be positive");
        }
        Ok(a)
    }
}

struct Timeline {
    heartbeats: Vec<Playhead>,
    coalesced: Vec<Playhead>,
    stale_in_buffer: u64,
    profiles: Vec<String>,
}

fn lcg(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed >> 33
}

fn simulate(a: &Args, base_ts: i64, tag: &str) -> Timeline {
    let mut seed = 42u64;
    let devices: Vec<(String, String, String, u64, i64, u32)> = (0..a.devices)
        .map(|d| {
            let profile = d / 2;
            let slot = d % 2;
            let title = if slot == 1 && profile % 2 == 1 { format!("title-solo-{}", profile % 40) } else { format!("title-{}", profile % 50) };
            let skew = match (slot, profile % 4) {
                (1, 0) => 5_333,
                (1, 2) => -5_333,
                _ => 0,
            };
            let offset = lcg(&mut seed) % a.heartbeat;
            let start = (lcg(&mut seed) % 1500) as u32;
            (format!("lg{base_ts}-{tag}-p{profile}"), format!("dev-{d}"), title, offset, skew, start)
        })
        .collect();
    let coalescer = Coalescer::new();
    let mut heartbeats = Vec::new();
    let mut coalesced = Vec::new();
    let total = a.minutes * 60;
    for sec in 0..=total {
        for (profile, device, title, offset, skew, start) in &devices {
            if (sec + offset) % a.heartbeat == 0 {
                let p = Playhead {
                    user_id: format!("user-{profile}"),
                    profile_id: profile.clone(),
                    device_id: device.clone(),
                    title_id: title.clone(),
                    episode_id: "s1e1".into(),
                    position_secs: start + sec as u32,
                    duration_secs: 3600,
                    client_ts_ms: base_ts + (sec as i64) * 1000 + skew,
                };
                coalescer.offer(p.clone());
                heartbeats.push(p);
            }
        }
        if sec > 0 && sec % a.flush == 0 {
            coalesced.extend(coalescer.drain());
        }
    }
    coalesced.extend(coalescer.drain());
    let mut profiles: Vec<String> = devices.iter().map(|d| d.0.clone()).collect();
    profiles.dedup();
    Timeline { heartbeats, coalesced, stale_in_buffer: coalescer.stats().stale, profiles }
}

fn expected(heartbeats: &[Playhead]) -> HashMap<(String, String), Playhead> {
    let mut winners: HashMap<(String, String), Playhead> = HashMap::new();
    for p in heartbeats {
        if winners.get(&p.key()).is_none_or(|cur| p.wins_over(cur)) {
            winners.insert(p.key(), p.clone());
        }
    }
    winners
}

struct Measure {
    ops: usize,
    secs: f64,
    p50: f64,
    p99: f64,
}

async fn run_writes(store: Arc<dyn PlayheadStore>, ops: Arc<Vec<Playhead>>, concurrency: usize) -> anyhow::Result<Measure> {
    let started = Instant::now();
    let mut tasks = Vec::new();
    for worker in 0..concurrency {
        let store = store.clone();
        let ops = ops.clone();
        tasks.push(tokio::spawn(async move {
            let mut lat = Vec::new();
            for p in ops.iter().skip(worker).step_by(concurrency) {
                let t = Instant::now();
                store.upsert(p).await?;
                lat.push(t.elapsed().as_micros() as f64 / 1000.0);
            }
            anyhow::Ok(lat)
        }));
    }
    let mut all = Vec::with_capacity(ops.len());
    for t in tasks {
        all.extend(t.await??);
    }
    let secs = started.elapsed().as_secs_f64();
    all.sort_by(f64::total_cmp);
    let pick = |q: f64| all.get(((all.len() as f64 - 1.0) * q) as usize).copied().unwrap_or(0.0);
    Ok(Measure { ops: all.len(), secs, p50: pick(0.50), p99: pick(0.99) })
}

async fn verify(store: &dyn PlayheadStore, profiles: &[String], want: &HashMap<(String, String), Playhead>) -> anyhow::Result<usize> {
    let mut mismatches = 0;
    let mut seen = 0;
    for profile in profiles {
        for got in store.list(profile).await? {
            seen += 1;
            if want.get(&got.key()) != Some(&got) {
                mismatches += 1;
            }
        }
    }
    Ok(mismatches + want.len().abs_diff(seen))
}

async fn clear(store: &dyn PlayheadStore, profiles: &[String]) -> anyhow::Result<()> {
    for p in profiles {
        store.clear(p).await?;
    }
    Ok(())
}

async fn bench(name: &str, store: Arc<dyn PlayheadStore>, a: &Args, base_ts: i64) -> anyhow::Result<()> {
    for mode in ["raw", "coalesced"] {
        let t = simulate(a, base_ts, &format!("{name}-{mode}"));
        let want = expected(&t.heartbeats);
        let ops = if mode == "raw" { t.heartbeats } else { t.coalesced };
        let m = run_writes(store.clone(), Arc::new(ops), a.concurrency).await?;
        let bad = verify(store.as_ref(), &t.profiles, &want).await?;
        println!(
            "| {name} | {mode} | {} | {:.2} | {:.0} | {:.3} | {:.3} | {} |",
            m.ops,
            m.secs,
            m.ops as f64 / m.secs,
            m.p50,
            m.p99,
            if bad == 0 { "yes".to_string() } else { format!("NO ({bad} wrong)") }
        );
        if bad != 0 {
            bail!("{name} {mode}: final state differs from last-write-wins expectation");
        }
        clear(store.as_ref(), &t.profiles).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let a = Args::parse()?;
    let base_ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
    let t = simulate(&a, base_ts, "plan");
    let want = expected(&t.heartbeats);
    println!("devices={} profiles={} minutes={} heartbeat={}s flush={}s concurrency={}", a.devices, t.profiles.len(), a.minutes, a.heartbeat, a.flush, a.concurrency);
    println!();
    println!("| heartbeats | stale dropped in buffer | writes after coalescing | write reduction | distinct rows |");
    println!("|---|---|---|---|---|");
    println!(
        "| {} | {} | {} | {:.1}x ({:.1}% fewer) | {} |",
        t.heartbeats.len(),
        t.stale_in_buffer,
        t.coalesced.len(),
        t.heartbeats.len() as f64 / t.coalesced.len() as f64,
        100.0 * (1.0 - t.coalesced.len() as f64 / t.heartbeats.len() as f64),
        want.len()
    );
    println!();
    println!("| store | mode | writes | seconds | writes/s | p50 ms | p99 ms | final rows match LWW |");
    println!("|---|---|---|---|---|---|---|---|");
    let redis: Arc<dyn PlayheadStore> = store::connect("redis", &a.redis, &a.scylla).await.context("redis")?.into();
    bench("redis", redis, &a, base_ts).await?;
    let scylla: Arc<dyn PlayheadStore> = store::connect("scylla", &a.redis, &a.scylla).await.context("scylla")?.into();
    bench("scylla", scylla, &a, base_ts).await?;
    Ok(())
}
