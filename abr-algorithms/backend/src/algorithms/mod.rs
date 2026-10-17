mod bola;
mod dynamic;
mod throughput;

pub use bola::Bola;
pub use dynamic::Dynamic;
pub use throughput::{Throughput, harmonic_mean};

use crate::AbrError;
use serde::Serialize;

pub struct Decision<'a> {
    pub ladder_kbps: &'a [f64],
    pub segment_s: f64,
    pub max_buffer_s: f64,
    pub buffer_s: f64,
    pub throughput_kbps: &'a [f64],
    pub last_quality: Option<usize>,
}

pub trait AbrAlgorithm {
    fn id(&self) -> &'static str;
    fn choose(&mut self, decision: &Decision) -> usize;
}

#[derive(Debug, Clone, Serialize)]
pub struct AlgorithmInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

pub fn catalog() -> Vec<AlgorithmInfo> {
    vec![
        AlgorithmInfo {
            id: throughput::ID,
            name: "Throughput",
            description: "Harmonic mean of the last 5 segment throughputs times 0.9, highest bitrate that fits",
        },
        AlgorithmInfo {
            id: bola::ID,
            name: "BOLA",
            description: "Buffer based Lyapunov utility maximisation (BOLA-BASIC), ignores throughput",
        },
        AlgorithmInfo {
            id: dynamic::ID,
            name: "Dynamic",
            description: "Throughput rule until the buffer reaches 10s, BOLA after, back below 5s",
        },
    ]
}

pub fn build(id: &str) -> Result<Box<dyn AbrAlgorithm>, AbrError> {
    match id {
        throughput::ID => Ok(Box::new(Throughput::default())),
        bola::ID => Ok(Box::new(Bola::default())),
        dynamic::ID => Ok(Box::new(Dynamic::default())),
        other => Err(AbrError::UnknownAlgorithm(other.to_string())),
    }
}

pub fn highest_below(ladder_kbps: &[f64], budget_kbps: f64) -> usize {
    ladder_kbps.iter().rposition(|b| *b <= budget_kbps).unwrap_or(0)
}
