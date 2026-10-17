use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct OriginConfig {
    pub latency_ms: u64,
    pub segment_bytes: usize,
    pub max_age: u64,
    pub stale_while_revalidate: u64,
}

pub struct OriginState {
    config: OriginConfig,
    requests: AtomicU64,
    versions: Mutex<HashMap<String, u64>>,
}

#[derive(Debug, Serialize)]
pub struct OriginStats {
    pub requests: u64,
    pub unique_keys: usize,
    pub config: OriginConfig,
}

impl OriginState {
    pub fn new(config: OriginConfig) -> Arc<Self> {
        Arc::new(OriginState {
            config,
            requests: AtomicU64::new(0),
            versions: Mutex::new(HashMap::new()),
        })
    }

    pub fn stats(&self) -> OriginStats {
        OriginStats {
            requests: self.requests.load(Ordering::Relaxed),
            unique_keys: self.versions.lock().map(|v| v.len()).unwrap_or(0),
            config: self.config.clone(),
        }
    }

    fn next_version(&self, key: &str) -> u64 {
        self.versions
            .lock()
            .map(|mut v| {
                let slot = v.entry(key.to_string()).or_insert(0);
                *slot += 1;
                *slot
            })
            .unwrap_or(0)
    }

    fn reset(&self) {
        self.requests.store(0, Ordering::Relaxed);
        if let Ok(mut v) = self.versions.lock() {
            v.clear();
        }
    }
}

pub fn segment_body(key: &str, version: u64, size: usize) -> Vec<u8> {
    let stamp = format!("{key} v{version}\n");
    stamp.bytes().cycle().take(size.max(stamp.len())).collect()
}

async fn segment(State(state): State<Arc<OriginState>>, Path(key): Path<String>) -> impl IntoResponse {
    state.requests.fetch_add(1, Ordering::Relaxed);
    tokio::time::sleep(Duration::from_millis(state.config.latency_ms)).await;
    let version = state.next_version(&key);
    let cache_control = format!(
        "public, max-age={}, stale-while-revalidate={}",
        state.config.max_age, state.config.stale_while_revalidate
    );
    (
        [
            (header::CONTENT_TYPE, "video/mp2t".to_string()),
            (header::CACHE_CONTROL, cache_control),
            (header::HeaderName::from_static("x-origin-version"), version.to_string()),
        ],
        segment_body(&key, version, state.config.segment_bytes),
    )
}

async fn stats(State(state): State<Arc<OriginState>>) -> Json<OriginStats> {
    Json(state.stats())
}

#[derive(Debug, Serialize)]
pub struct KeyStats {
    pub key: String,
    pub requests: u64,
}

async fn key_stats(State(state): State<Arc<OriginState>>, Path(key): Path<String>) -> Json<KeyStats> {
    let requests = state.versions.lock().ok().and_then(|v| v.get(&key).copied()).unwrap_or(0);
    Json(KeyStats { key, requests })
}

async fn reset(State(state): State<Arc<OriginState>>) -> Json<OriginStats> {
    state.reset();
    Json(state.stats())
}

pub fn router(state: Arc<OriginState>) -> Router {
    Router::new()
        .route("/seg/{key}", get(segment))
        .route("/stats", get(stats))
        .route("/stats/reset", post(reset))
        .route("/stats/key/{key}", get(key_stats))
        .route("/health", get(|| async { "ok" }))
        .with_state(state)
}
