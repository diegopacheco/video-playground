#![allow(dead_code)]

use chrono::{DateTime, TimeZone, Utc};
use simulcast_scheduler::clock::{ManualClock, OffsetClock};
use simulcast_scheduler::geo::HeaderThenIpResolver;
use simulcast_scheduler::herd::jitter::Jitter;
use simulcast_scheduler::model::{Episode, RegionRule};
use simulcast_scheduler::service::{HerdConfig, Scheduler};
use simulcast_scheduler::store::InMemoryStore;
use std::collections::BTreeMap;
use std::sync::Arc;

pub fn utc(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, mo, d, h, mi, s).single().expect("valid instant")
}

pub fn release() -> DateTime<Utc> {
    utc(2026, 11, 1, 6, 0, 0)
}

pub fn episode() -> Episode {
    Episode {
        id: "ep".into(),
        title: "Ep".into(),
        release_utc: release(),
        default_rule: RegionRule::open(),
        regions: BTreeMap::from([
            ("CN".into(), RegionRule::Blocked),
            ("IN".into(), RegionRule::Window { delay_secs: 3_600, until: None }),
            (
                "KR".into(),
                RegionRule::Window { delay_secs: 0, until: Some(utc(2026, 11, 8, 6, 0, 0)) },
            ),
        ]),
    }
}

pub struct Rig {
    pub clock: Arc<ManualClock>,
    pub scheduler: Arc<Scheduler>,
}

pub fn rig(start: DateTime<Utc>, config: HerdConfig) -> Rig {
    let clock = Arc::new(ManualClock::new(start));
    let scheduler = Arc::new(Scheduler::new(
        Arc::new(InMemoryStore::default()),
        Arc::new(HeaderThenIpResolver::with_default_table()),
        Arc::new(OffsetClock::new(clock.clone())),
        Jitter::seeded(42),
        config,
    ));
    scheduler.upsert(episode()).expect("seed episode");
    Rig { clock, scheduler }
}
