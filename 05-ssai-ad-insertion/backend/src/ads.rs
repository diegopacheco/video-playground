use serde::Serialize;

use crate::error::{Result, SsaiError};
use crate::users::{Tier, UserProfile};

pub const EPSILON: f64 = 0.001;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AdSegment {
    pub duration: f64,
    pub file: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Creative {
    pub id: String,
    pub advertiser: String,
    pub regions: Vec<String>,
    pub segments: Vec<AdSegment>,
    pub tracked: bool,
}

impl Creative {
    pub fn duration(&self) -> f64 {
        self.segments.iter().map(|s| s.duration).sum()
    }

    pub fn targets(&self, region: &str) -> bool {
        self.regions.is_empty() || self.regions.iter().any(|r| r == region)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BreakInfo {
    pub id: String,
    pub duration: f64,
}

pub trait AdDecisionService: Send + Sync {
    fn decide(&self, user: &UserProfile, brk: &BreakInfo) -> Result<Vec<Creative>>;
}

pub struct Catalog {
    pub creatives: Vec<Creative>,
    pub slate: Creative,
}

impl Catalog {
    pub fn find(&self, id: &str) -> Result<&Creative> {
        std::iter::once(&self.slate)
            .chain(self.creatives.iter())
            .find(|c| c.id == id)
            .ok_or_else(|| SsaiError::UnknownCreative(id.into()))
    }
}

pub struct TargetedAdDecision {
    catalog: std::sync::Arc<Catalog>,
}

impl TargetedAdDecision {
    pub fn new(catalog: std::sync::Arc<Catalog>) -> TargetedAdDecision {
        TargetedAdDecision { catalog }
    }
}

impl AdDecisionService for TargetedAdDecision {
    fn decide(&self, user: &UserProfile, brk: &BreakInfo) -> Result<Vec<Creative>> {
        if user.tier == Tier::Premium {
            return Ok(Vec::new());
        }
        let mut eligible: Vec<&Creative> = self.catalog.creatives.iter().filter(|c| c.targets(&user.region)).collect();
        if !eligible.is_empty() {
            let offset = rotation(&user.id, &brk.id) % eligible.len();
            eligible.rotate_left(offset);
        }
        let mut remaining = brk.duration;
        let mut pod = Vec::new();
        for creative in eligible {
            if creative.duration() <= remaining + EPSILON {
                remaining -= creative.duration();
                pod.push(creative.clone());
            }
        }
        let slate = &self.catalog.slate;
        if slate.duration() <= EPSILON {
            return Err(SsaiError::InvalidCatalog("slate has no duration".into()));
        }
        while remaining + EPSILON >= slate.duration() {
            remaining -= slate.duration();
            pod.push(slate.clone());
        }
        Ok(pod)
    }
}

fn rotation(user: &str, brk: &str) -> usize {
    user.bytes().chain(brk.bytes()).map(usize::from).sum()
}
