use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RegionRule {
    Window {
        delay_secs: i64,
        until: Option<DateTime<Utc>>,
    },
    Blocked,
}

impl RegionRule {
    pub fn open() -> Self {
        RegionRule::Window { delay_secs: 0, until: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub title: String,
    pub release_utc: DateTime<Utc>,
    pub default_rule: RegionRule,
    pub regions: BTreeMap<String, RegionRule>,
}
