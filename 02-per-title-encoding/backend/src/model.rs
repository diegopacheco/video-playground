use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn label(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rendition {
    pub resolution: Resolution,
    pub kbps: u32,
}

impl Rendition {
    pub fn file_name(&self) -> String {
        format!("{}_{}k.mp4", self.resolution.label(), self.kbps)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub resolution: Resolution,
    pub target_kbps: u32,
    pub bitrate_kbps: f64,
    pub scores: BTreeMap<String, f64>,
}

impl Point {
    pub fn score(&self, metric: &str) -> f64 {
        self.scores.get(metric).copied().unwrap_or(0.0)
    }

    pub fn is(&self, rendition: &Rendition) -> bool {
        self.resolution == rendition.resolution && self.target_kbps == rendition.kbps
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LadderRung {
    pub target: f64,
    pub reached: bool,
    pub point: Point,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericComparison {
    pub point: Point,
    pub equal_quality_kbps: Option<f64>,
    pub saving_pct: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Savings {
    pub generic_kbps: f64,
    pub per_title_kbps: f64,
    pub saving_pct: f64,
    pub compared_rungs: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitleReport {
    pub id: String,
    pub name: String,
    pub description: String,
    pub points: Vec<Point>,
    pub hulls: BTreeMap<String, Vec<Point>>,
    pub ladder: Vec<LadderRung>,
    pub generic: Vec<GenericComparison>,
    pub savings: Savings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    pub generated_at: u64,
    pub elapsed_secs: f64,
    pub encoder: String,
    pub metrics: Vec<String>,
    pub ladder_metric: String,
    pub targets: Vec<f64>,
    pub titles: Vec<TitleReport>,
}
