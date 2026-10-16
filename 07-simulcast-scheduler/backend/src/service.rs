use crate::clock::{Clock, OffsetClock};
use crate::entitlement::{self, BlockReason, Status};
use crate::error::AppError;
use crate::geo::{self, COUNTRIES, GeoResolver, UNKNOWN_COUNTRY};
use crate::herd::bucket::TokenBucket;
use crate::herd::cache::{Entry, ResponseCache};
use crate::herd::jitter::Jitter;
use crate::herd::metrics::{Metrics, MetricsSnapshot, bump};
use crate::model::Episode;
use crate::store::ScheduleStore;
use crate::tzdisplay::{LocalTime, local_time};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy)]
pub struct HerdConfig {
    pub ttl_ms: u64,
    pub ttl_jitter_pct: u64,
    pub retry_base_ms: u64,
    pub retry_jitter_ms: u64,
    pub unlock_spread_ms: u64,
    pub warm_horizon_ms: i64,
    pub bucket_capacity: f64,
    pub bucket_refill_per_sec: f64,
}

impl Default for HerdConfig {
    fn default() -> Self {
        Self {
            ttl_ms: 5_000,
            ttl_jitter_pct: 20,
            retry_base_ms: 1_000,
            retry_jitter_ms: 2_000,
            unlock_spread_ms: 3_000,
            warm_horizon_ms: 2_000,
            bucket_capacity: 50.0,
            bucket_refill_per_sec: 200.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EntitlementView {
    pub episode_id: String,
    pub country: String,
    pub status: &'static str,
    pub unlock_at: Option<DateTime<Utc>>,
    pub available_until: Option<DateTime<Utc>>,
    pub countdown_ms: Option<i64>,
    pub reason: Option<BlockReason>,
    pub local_release: Option<LocalTime>,
    pub server_now: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Served {
        view: Box<EntitlementView>,
        cache_hit: bool,
        retry_after_ms: Option<u64>,
    },
    Throttled {
        retry_after_ms: u64,
    },
}

#[derive(Debug, Clone)]
pub enum PlayOutcome {
    Granted { session_id: String, view: EntitlementView },
    Locked { retry_after_ms: u64, view: EntitlementView },
    Blocked { view: EntitlementView },
    Throttled { retry_after_ms: u64 },
}

#[derive(Debug, Clone, Serialize)]
pub struct EpisodeBoard {
    pub id: String,
    pub title: String,
    pub release_utc: DateTime<Utc>,
    pub regions: Vec<EntitlementView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Board {
    pub server_now: DateTime<Utc>,
    pub offset_ms: i64,
    pub episodes: Vec<EpisodeBoard>,
}

pub struct Scheduler {
    store: Arc<dyn ScheduleStore>,
    geo: Arc<dyn GeoResolver>,
    clock: Arc<OffsetClock>,
    cache: ResponseCache,
    bucket: TokenBucket,
    jitter: Jitter,
    metrics: Metrics,
    sessions: AtomicU64,
    config: HerdConfig,
}

impl Scheduler {
    pub fn new(
        store: Arc<dyn ScheduleStore>,
        geo: Arc<dyn GeoResolver>,
        clock: Arc<OffsetClock>,
        jitter: Jitter,
        config: HerdConfig,
    ) -> Self {
        let bucket = TokenBucket::new(
            config.bucket_capacity,
            config.bucket_refill_per_sec,
            clock.now().timestamp_millis(),
        );
        Self {
            store,
            geo,
            clock,
            cache: ResponseCache::default(),
            bucket,
            jitter,
            metrics: Metrics::default(),
            sessions: AtomicU64::new(0),
            config,
        }
    }

    pub fn now(&self) -> DateTime<Utc> {
        self.clock.now()
    }

    pub fn offset_ms(&self) -> i64 {
        self.clock.offset_ms()
    }

    pub fn set_offset_ms(&self, offset_ms: i64) {
        self.clock.set_offset_ms(offset_ms);
    }

    pub fn resolve_country(&self, header: Option<&str>, ip: Option<IpAddr>) -> String {
        self.geo.resolve(header, ip)
    }

    pub fn episodes(&self) -> Result<Vec<Episode>, AppError> {
        self.store.list()
    }

    pub fn upsert(&self, episode: Episode) -> Result<Episode, AppError> {
        self.store.upsert(episode.clone())?;
        self.cache.invalidate_episode(&episode.id);
        Ok(episode)
    }

    pub fn reschedule(&self, id: &str, release_in_secs: i64) -> Result<Episode, AppError> {
        let mut episode = self.store.get(id)?;
        let now_ms = self.now().timestamp_millis();
        let release_ms = (now_ms + release_in_secs * 1000) / 1000 * 1000;
        episode.release_utc = DateTime::from_timestamp_millis(release_ms)
            .ok_or_else(|| AppError::InvalidSchedule("release out of range".into()))?;
        self.upsert(episode)
    }

    pub fn metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot(self.cache.len(), self.bucket.tokens())
    }

    pub fn reset_metrics(&self) {
        self.metrics.reset();
    }

    fn compute(&self, episode_id: &str, country: &str, now: DateTime<Utc>) -> Result<(Status, Option<DateTime<Utc>>), AppError> {
        let plan = self.store.plan(episode_id)?;
        if country == UNKNOWN_COUNTRY {
            return Ok((Status::GeoBlocked { reason: BlockReason::UnknownRegion }, None));
        }
        let access = plan.access(country);
        Ok((entitlement::evaluate(access, now), entitlement::next_transition(access, now)))
    }

    fn jittered_expiry(&self, start: DateTime<Utc>, bound: Option<DateTime<Utc>>) -> DateTime<Utc> {
        let ttl = self.jitter.spread(self.config.ttl_ms, self.config.ttl_jitter_pct) as i64;
        let expiry = start + Duration::milliseconds(ttl);
        bound.map_or(expiry, |b| expiry.min(b))
    }

    pub fn view(&self, episode_id: &str, country: &str, status: Status, now: DateTime<Utc>) -> Result<EntitlementView, AppError> {
        let (unlock_at, until, reason) = match status {
            Status::Available { since, until } => (Some(since), until, None),
            Status::Locked { unlock_at } => (Some(unlock_at), None, None),
            Status::GeoBlocked { reason } => (None, None, Some(reason)),
        };
        let countdown_ms = match status {
            Status::Locked { unlock_at } => Some((unlock_at - now).num_milliseconds()),
            _ => None,
        };
        let local_release = unlock_at
            .map(|at| local_time(at, geo::timezone_for(country)))
            .transpose()?;
        Ok(EntitlementView {
            episode_id: episode_id.to_string(),
            country: country.to_string(),
            status: status.label(),
            unlock_at,
            available_until: until,
            countdown_ms,
            reason,
            local_release,
            server_now: now,
        })
    }

    fn throttle_delay(&self) -> u64 {
        self.config.retry_base_ms + self.jitter.below(self.config.retry_jitter_ms + 1)
    }

    pub fn locked_retry_after(&self, countdown_ms: i64) -> u64 {
        countdown_ms.max(0) as u64 + self.jitter.below(self.config.unlock_spread_ms + 1)
    }

    pub fn entitlement(&self, episode_id: &str, country: &str) -> Result<Decision, AppError> {
        let now = self.now();
        let (status, cache_hit) = match self.cache.get(episode_id, country, now) {
            Some(status) => {
                bump(&self.metrics.cache_hits);
                (status, true)
            }
            None => {
                bump(&self.metrics.cache_misses);
                if !self.bucket.try_acquire(now.timestamp_millis()) {
                    bump(&self.metrics.throttled);
                    return Ok(Decision::Throttled { retry_after_ms: self.throttle_delay() });
                }
                let (status, next) = self.compute(episode_id, country, now)?;
                let entry = Entry { status, valid_from: now, expires_at: self.jittered_expiry(now, next) };
                self.cache.put(episode_id, country, entry, now);
                (status, false)
            }
        };
        let view = self.view(episode_id, country, status, now)?;
        let retry_after_ms = view.countdown_ms.map(|ms| self.locked_retry_after(ms));
        Ok(Decision::Served { view: Box::new(view), cache_hit, retry_after_ms })
    }

    pub fn play(&self, episode_id: &str, country: &str) -> Result<PlayOutcome, AppError> {
        let (view, retry_after_ms) = match self.entitlement(episode_id, country)? {
            Decision::Throttled { retry_after_ms } => return Ok(PlayOutcome::Throttled { retry_after_ms }),
            Decision::Served { view, retry_after_ms, .. } => (*view, retry_after_ms),
        };
        match view.status {
            "locked" => Ok(PlayOutcome::Locked { retry_after_ms: retry_after_ms.unwrap_or(0), view }),
            "geo_blocked" => Ok(PlayOutcome::Blocked { view }),
            _ if self.bucket.try_acquire(self.now().timestamp_millis()) => {
                bump(&self.metrics.plays_granted);
                let n = self.sessions.fetch_add(1, Ordering::Relaxed) + 1;
                Ok(PlayOutcome::Granted { session_id: format!("{episode_id}-{country}-{n}"), view })
            }
            _ => {
                bump(&self.metrics.plays_throttled);
                Ok(PlayOutcome::Throttled { retry_after_ms: self.throttle_delay() })
            }
        }
    }

    pub fn warm(&self) -> Result<usize, AppError> {
        let now = self.now();
        let horizon = now + Duration::milliseconds(self.config.warm_horizon_ms);
        let mut warmed = 0;
        for episode in self.store.list()? {
            let plan = self.store.plan(&episode.id)?;
            for (country, _) in COUNTRIES {
                let access = plan.access(country);
                let Some(at) = entitlement::next_transition(access, now) else { continue };
                if at > horizon {
                    continue;
                }
                let status = entitlement::evaluate(access, at);
                let following = entitlement::next_transition(access, at);
                let entry = Entry { status, valid_from: at, expires_at: self.jittered_expiry(at, following) };
                self.cache.put(&episode.id, country, entry, now);
                bump(&self.metrics.warmed);
                warmed += 1;
            }
        }
        Ok(warmed)
    }

    pub fn board(&self) -> Result<Board, AppError> {
        let now = self.now();
        let episodes = self
            .store
            .list()?
            .into_iter()
            .map(|episode| {
                let regions = COUNTRIES
                    .iter()
                    .map(|(country, _)| {
                        let (status, _) = self.compute(&episode.id, country, now)?;
                        self.view(&episode.id, country, status, now)
                    })
                    .collect::<Result<Vec<_>, AppError>>()?;
                Ok(EpisodeBoard {
                    id: episode.id,
                    title: episode.title,
                    release_utc: episode.release_utc,
                    regions,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;
        Ok(Board { server_now: now, offset_ms: self.offset_ms(), episodes })
    }
}
