use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Serialize;

use crate::coalescer::{Coalescer, CoalescerStats, Offer};
use crate::error::StoreError;
use crate::model::Playhead;
use crate::row::{RowItem, continue_watching};
use crate::store::PlayheadStore;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<dyn PlayheadStore>,
    pub coalescer: Arc<Coalescer>,
    pub flush_secs: u64,
}

pub enum ApiError {
    BadRequest(String),
    Store(StoreError),
}

impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> Self {
        ApiError::Store(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            ApiError::Store(e) => (StatusCode::SERVICE_UNAVAILABLE, e.to_string()),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

#[derive(Serialize)]
pub struct OfferResponse {
    pub result: Offer,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub store: &'static str,
    pub flush_secs: u64,
    #[serde(flatten)]
    pub stats: CoalescerStats,
    pub reduction: f64,
}

#[derive(Serialize)]
pub struct FlushResponse {
    pub written: usize,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/heartbeat", post(heartbeat))
        .route("/api/profiles/{profile}/continue-watching", get(row))
        .route("/api/profiles/{profile}", delete(reset))
        .route("/api/stats", get(stats))
        .route("/api/flush", post(flush))
        .with_state(state)
}

async fn heartbeat(State(s): State<AppState>, Json(p): Json<Playhead>) -> Result<Json<OfferResponse>, ApiError> {
    p.validate().map_err(ApiError::BadRequest)?;
    Ok(Json(OfferResponse { result: s.coalescer.offer(p) }))
}

async fn row(State(s): State<AppState>, Path(profile): Path<String>) -> Result<Json<Vec<RowItem>>, ApiError> {
    let stored = s.store.list(&profile).await?;
    Ok(Json(continue_watching(stored, s.coalescer.pending_for(&profile))))
}

async fn reset(State(s): State<AppState>, Path(profile): Path<String>) -> Result<StatusCode, ApiError> {
    s.coalescer.forget_profile(&profile);
    s.store.clear(&profile).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn stats(State(s): State<AppState>) -> Json<StatsResponse> {
    let stats = s.coalescer.stats();
    let reduction = if stats.writes == 0 { 0.0 } else { stats.heartbeats as f64 / stats.writes as f64 };
    Json(StatsResponse { store: s.store.name(), flush_secs: s.flush_secs, stats, reduction })
}

async fn flush(State(s): State<AppState>) -> Result<Json<FlushResponse>, ApiError> {
    let written = s.coalescer.flush(s.store.as_ref()).await?;
    Ok(Json(FlushResponse { written }))
}
