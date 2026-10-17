use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use crate::HerdError;
use crate::room::{JoinResponse, LiveRoom, PlayResponse, RoomStats, StatusResponse};
use crate::sim::{SimConfig, SimReport, all_strategies, simulate, strategy_by_name};

pub struct AppState {
    room: Mutex<LiveRoom>,
    rate_per_s: u64,
    epoch: Instant,
}

pub type Shared = Arc<AppState>;

impl AppState {
    pub fn new(rate_per_s: u64) -> Result<Shared, HerdError> {
        Ok(Arc::new(Self { room: Mutex::new(LiveRoom::new(rate_per_s, 13)?), rate_per_s, epoch: Instant::now() }))
    }

    fn now_us(&self) -> u64 {
        self.epoch.elapsed().as_micros() as u64
    }

    fn with_room<T>(&self, f: impl FnOnce(&mut LiveRoom, u64) -> T) -> Result<T, HerdError> {
        let mut room = self.room.lock().map_err(|e| HerdError::Worker(e.to_string()))?;
        let now = self.now_us();
        Ok(f(&mut room, now))
    }
}

impl IntoResponse for HerdError {
    fn into_response(self) -> Response {
        let status = match self {
            HerdError::UnknownStrategy(_) | HerdError::InvalidConfig(_) => StatusCode::BAD_REQUEST,
            HerdError::TicketNotFound(_) => StatusCode::NOT_FOUND,
            HerdError::NotAdmitted => StatusCode::FORBIDDEN,
            HerdError::Worker(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(serde_json::json!({ "error": self.to_string() }))).into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    pub strategy: String,
    #[serde(default)]
    pub config: SimConfig,
}

#[derive(Debug, Deserialize)]
pub struct CompareRequest {
    #[serde(default)]
    pub config: SimConfig,
}

#[derive(Debug, Deserialize)]
pub struct PlayQuery {
    pub token: String,
}

pub fn router(state: Shared) -> Router {
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/strategies", get(strategies))
        .route("/api/config/default", get(|| async { Json(SimConfig::default()) }))
        .route("/api/simulate", post(simulate_one))
        .route("/api/compare", post(compare))
        .route("/room/join", post(join))
        .route("/room/status/{ticket}", get(status))
        .route("/room/play", get(play))
        .route("/room/stats", get(stats))
        .route("/room/reset", post(reset))
        .with_state(state)
}

async fn strategies() -> Json<Vec<&'static str>> {
    Json(all_strategies(&SimConfig::default()).iter().map(|s| s.name()).collect())
}

async fn simulate_one(Json(req): Json<SimulateRequest>) -> Result<Json<SimReport>, HerdError> {
    req.config.validate()?;
    let report = tokio::task::spawn_blocking(move || {
        let strategy = strategy_by_name(&req.strategy, &req.config)?;
        simulate(strategy.as_ref(), &req.config)
    })
    .await
    .map_err(|e| HerdError::Worker(e.to_string()))??;
    Ok(Json(report))
}

pub fn compare_all(cfg: &SimConfig) -> Result<Vec<SimReport>, HerdError> {
    cfg.validate()?;
    let strategies = all_strategies(cfg);
    std::thread::scope(|scope| {
        let handles: Vec<_> = strategies.iter().map(|s| scope.spawn(move || simulate(s.as_ref(), cfg))).collect();
        handles
            .into_iter()
            .map(|h| h.join().map_err(|_| HerdError::Worker("simulation thread panicked".into()))?)
            .collect()
    })
}

async fn compare(Json(req): Json<CompareRequest>) -> Result<Json<Vec<SimReport>>, HerdError> {
    let reports = tokio::task::spawn_blocking(move || compare_all(&req.config))
        .await
        .map_err(|e| HerdError::Worker(e.to_string()))??;
    Ok(Json(reports))
}

async fn join(State(state): State<Shared>) -> Result<Json<JoinResponse>, HerdError> {
    Ok(Json(state.with_room(|room, now| room.join(now))?))
}

async fn status(State(state): State<Shared>, Path(ticket): Path<u64>) -> Result<Json<StatusResponse>, HerdError> {
    Ok(Json(state.with_room(|room, now| room.status(ticket, now))??))
}

async fn play(State(state): State<Shared>, Query(q): Query<PlayQuery>) -> Result<Json<PlayResponse>, HerdError> {
    Ok(Json(state.with_room(|room, now| room.play(&q.token, now))??))
}

async fn stats(State(state): State<Shared>) -> Result<Json<RoomStats>, HerdError> {
    Ok(Json(state.with_room(|room, now| room.stats(now))?))
}

async fn reset(State(state): State<Shared>) -> Result<Json<RoomStats>, HerdError> {
    let fresh = LiveRoom::new(state.rate_per_s, 13)?;
    Ok(Json(state.with_room(|room, now| {
        *room = fresh;
        room.stats(now)
    })?))
}
