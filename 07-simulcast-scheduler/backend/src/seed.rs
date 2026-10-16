use crate::model::{Episode, RegionRule};
use chrono::{DateTime, Duration, TimeZone, Utc};
use std::collections::BTreeMap;

fn window(delay_secs: i64, until: Option<DateTime<Utc>>) -> RegionRule {
    RegionRule::Window { delay_secs, until }
}

pub fn episodes(now: DateTime<Utc>) -> Vec<Episode> {
    let neon_release = now + Duration::seconds(120);
    let ember_release = now - Duration::days(2);
    let glass_release = Utc.with_ymd_and_hms(2026, 11, 1, 5, 30, 0).single().unwrap_or(now);
    vec![
        Episode {
            id: "neon-harbor-s1e1".into(),
            title: "Neon Harbor S1E1".into(),
            release_utc: neon_release,
            default_rule: RegionRule::open(),
            regions: BTreeMap::from([
                ("IN".into(), window(3_600, None)),
                ("AU".into(), window(86_400, None)),
                ("CN".into(), RegionRule::Blocked),
                ("KR".into(), window(0, Some(neon_release + Duration::days(7)))),
            ]),
        },
        Episode {
            id: "glass-orbit-s2e5".into(),
            title: "Glass Orbit S2E5".into(),
            release_utc: glass_release,
            default_rule: RegionRule::open(),
            regions: BTreeMap::from([
                ("DE".into(), RegionRule::Blocked),
                ("GB".into(), window(1_800, None)),
                ("JP".into(), window(43_200, None)),
            ]),
        },
        Episode {
            id: "ember-road-s3e2".into(),
            title: "Ember Road S3E2".into(),
            release_utc: ember_release,
            default_rule: RegionRule::open(),
            regions: BTreeMap::from([
                ("BR".into(), RegionRule::Blocked),
                ("JP".into(), window(0, Some(now - Duration::hours(1)))),
                ("AU".into(), window(259_200, None)),
            ]),
        },
    ]
}
