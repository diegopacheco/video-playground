use crate::beacon::{Beacon, BeaconKind};

pub const TITLES: [&str; 4] = ["big-heist", "ocean-deep", "mars-colony", "jazz-nights"];
pub const LADDER_KBPS: [u32; 4] = [800, 1800, 3500, 6000];

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo + 1)
    }

    pub fn chance(&mut self, p: f64) -> bool {
        self.unit() < p
    }
}

#[derive(Debug, Clone)]
pub struct CdnProfile {
    pub name: &'static str,
    pub startup_ms: (u64, u64),
    pub rebuffer_chance: f64,
    pub rebuffer_ms: (u64, u64),
    pub switch_chance: f64,
    pub error_chance: f64,
}

pub fn profiles() -> Vec<CdnProfile> {
    vec![
        CdnProfile {
            name: "cdn-alpha",
            startup_ms: (400, 1_200),
            rebuffer_chance: 0.02,
            rebuffer_ms: (100, 400),
            switch_chance: 0.04,
            error_chance: 0.001,
        },
        CdnProfile {
            name: "cdn-beta",
            startup_ms: (500, 1_500),
            rebuffer_chance: 0.03,
            rebuffer_ms: (100, 500),
            switch_chance: 0.05,
            error_chance: 0.002,
        },
        CdnProfile {
            name: "cdn-gamma",
            startup_ms: (2_500, 7_000),
            rebuffer_chance: 0.25,
            rebuffer_ms: (300, 900),
            switch_chance: 0.2,
            error_chance: 0.02,
        },
    ]
}

struct SynthSession {
    id: String,
    title: &'static str,
    profile: usize,
    level: usize,
    started: bool,
    remaining_secs: u64,
}

pub struct Fleet {
    rng: Rng,
    profiles: Vec<CdnProfile>,
    sessions: Vec<SynthSession>,
    next_id: u64,
}

impl Fleet {
    pub fn new(size: usize, seed: u64) -> Self {
        let mut fleet = Self {
            rng: Rng::new(seed),
            profiles: profiles(),
            sessions: Vec::with_capacity(size),
            next_id: 0,
        };
        for _ in 0..size {
            let session = fleet.spawn();
            fleet.sessions.push(session);
        }
        fleet
    }

    fn spawn(&mut self) -> SynthSession {
        self.next_id += 1;
        SynthSession {
            id: format!("synth-{}-{}", self.next_id, self.rng.next_u64() % 100_000),
            title: TITLES[self.rng.range(0, TITLES.len() as u64 - 1) as usize],
            profile: self.rng.range(0, self.profiles.len() as u64 - 1) as usize,
            level: 1,
            started: false,
            remaining_secs: self.rng.range(20, 90),
        }
    }

    pub fn tick(&mut self, now_ms: i64) -> Vec<Beacon> {
        let mut out = Vec::new();
        for idx in 0..self.sessions.len() {
            let ended = self.step(idx, now_ms, &mut out);
            if ended {
                self.sessions[idx] = self.spawn();
            }
        }
        out
    }

    fn step(&mut self, idx: usize, now_ms: i64, out: &mut Vec<Beacon>) -> bool {
        let profile = self.profiles[self.sessions[idx].profile].clone();
        let rng = &mut self.rng;
        let session = &mut self.sessions[idx];
        let (id, title) = (session.id.clone(), session.title);
        let beacon = |event: BeaconKind| Beacon {
            session_id: id.clone(),
            title: title.to_string(),
            cdn: profile.name.to_string(),
            ts: now_ms,
            event,
        };
        if !session.started {
            let startup_ms = rng.range(profile.startup_ms.0, profile.startup_ms.1);
            out.push(beacon(BeaconKind::Startup { startup_ms }));
            session.started = true;
            return false;
        }
        let mut stalled = 0;
        if rng.chance(profile.rebuffer_chance) {
            stalled = rng.range(profile.rebuffer_ms.0, profile.rebuffer_ms.1);
            out.push(beacon(BeaconKind::RebufferStart));
            out.push(beacon(BeaconKind::RebufferEnd { rebuffer_ms: stalled }));
        }
        if rng.chance(profile.switch_chance) {
            let from = session.level;
            let to = if stalled > 0 || (from + 1 == LADDER_KBPS.len()) {
                from.saturating_sub(1)
            } else {
                from + 1
            };
            if to != from {
                out.push(beacon(BeaconKind::BitrateSwitch {
                    from_kbps: LADDER_KBPS[from],
                    to_kbps: LADDER_KBPS[to],
                }));
                session.level = to;
            }
        }
        out.push(beacon(BeaconKind::Heartbeat {
            played_ms: 1_000 - stalled,
        }));
        if rng.chance(profile.error_chance) {
            out.push(beacon(BeaconKind::Error {
                code: "MEDIA_ERR_NETWORK".into(),
            }));
            return true;
        }
        session.remaining_secs = session.remaining_secs.saturating_sub(1);
        session.remaining_secs == 0
    }
}
