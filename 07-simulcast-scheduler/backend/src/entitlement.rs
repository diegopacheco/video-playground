use crate::error::AppError;
use crate::model::{Episode, RegionRule};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Window {
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    },
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockReason {
    NotLicensed,
    LicenseExpired,
    UnknownRegion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Available {
        since: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    },
    Locked {
        unlock_at: DateTime<Utc>,
    },
    GeoBlocked {
        reason: BlockReason,
    },
}

impl Status {
    pub fn label(&self) -> &'static str {
        match self {
            Status::Available { .. } => "available",
            Status::Locked { .. } => "locked",
            Status::GeoBlocked { .. } => "geo_blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlockPlan {
    pub episode_id: String,
    pub release_utc: DateTime<Utc>,
    default: Access,
    regions: BTreeMap<String, Access>,
}

impl UnlockPlan {
    pub fn build(episode: &Episode) -> Result<Self, AppError> {
        let default = to_access(episode.release_utc, &episode.default_rule)?;
        let regions = episode
            .regions
            .iter()
            .map(|(country, rule)| Ok((country.clone(), to_access(episode.release_utc, rule)?)))
            .collect::<Result<BTreeMap<_, _>, AppError>>()?;
        Ok(Self {
            episode_id: episode.id.clone(),
            release_utc: episode.release_utc,
            default,
            regions,
        })
    }

    pub fn access(&self, country: &str) -> Access {
        self.regions.get(country).copied().unwrap_or(self.default)
    }
}

fn to_access(release: DateTime<Utc>, rule: &RegionRule) -> Result<Access, AppError> {
    match rule {
        RegionRule::Blocked => Ok(Access::Blocked),
        RegionRule::Window { delay_secs, until } => {
            if *delay_secs < 0 {
                return Err(AppError::InvalidSchedule(format!("negative delay {delay_secs}")));
            }
            let from = release + Duration::seconds(*delay_secs);
            if until.is_some_and(|end| end <= from) {
                return Err(AppError::InvalidSchedule("window ends before it opens".into()));
            }
            Ok(Access::Window { from, until: *until })
        }
    }
}

pub fn evaluate(access: Access, now: DateTime<Utc>) -> Status {
    match access {
        Access::Blocked => Status::GeoBlocked { reason: BlockReason::NotLicensed },
        Access::Window { from, .. } if now < from => Status::Locked { unlock_at: from },
        Access::Window { until: Some(end), .. } if now >= end => Status::GeoBlocked {
            reason: BlockReason::LicenseExpired,
        },
        Access::Window { from, until } => Status::Available { since: from, until },
    }
}

pub fn next_transition(access: Access, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    match access {
        Access::Blocked => None,
        Access::Window { from, .. } if now < from => Some(from),
        Access::Window { until, .. } => until.filter(|end| now < *end),
    }
}
