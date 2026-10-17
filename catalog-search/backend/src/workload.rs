use std::time::Instant;

use serde::Serialize;

use crate::engine::DefaultEngine;
use crate::error::SearchError;

pub const QUERIES: &[&str] = &[
    "shingeki",
    "進撃",
    "atack on titan",
    "kimetsu no yaiba",
    "鬼滅",
    "kimet",
    "naruto",
    "なると",
    "one piece",
    "わんぴーす",
    "fullmetal alchemst",
    "hagane no renkinjutsushi",
    "death note",
    "デスノート",
    "my hero acadmia",
    "jujutsu",
    "呪術",
    "spirited away",
    "千と千尋",
    "kimi no na wa",
    "your name",
    "pokemon",
    "evangelion",
    "cowboy bebop",
    "steins gate",
    "ansatsu kyoshitsu",
    "frieren",
    "葬送",
    "the",
    "no",
    "a",
    "sword art",
    "ソードアート",
    "attack on ti",
    "tokyo",
    "東京",
    "zzzzqqq",
    "dragon ball",
    "haikyu",
    "violet evergarden",
];

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Latency {
    pub samples: usize,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub max_us: u64,
}

fn percentile(sorted: &[u64], p: f64) -> u64 {
    let rank = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[rank]
}

pub fn measure(engine: &DefaultEngine, rounds: usize) -> Result<Latency, SearchError> {
    let mut samples = Vec::with_capacity(rounds * QUERIES.len());
    for _ in 0..rounds {
        for q in QUERIES {
            let started = Instant::now();
            engine.search(q, 10)?;
            samples.push(started.elapsed().as_micros() as u64);
        }
    }
    samples.sort_unstable();
    Ok(Latency {
        samples: samples.len(),
        p50_us: percentile(&samples, 0.50),
        p95_us: percentile(&samples, 0.95),
        p99_us: percentile(&samples, 0.99),
        max_us: *samples.last().unwrap_or(&0),
    })
}
