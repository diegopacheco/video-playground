use crate::AbrError;
use crate::algorithms::{AlgorithmInfo, catalog};
use crate::player::PlayerConfig;
use crate::report::{SimulationReport, run};
use crate::trace::{Trace, TraceSummary};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::sync::Arc;

pub struct AppState {
    pub traces: Vec<Trace>,
    pub config: PlayerConfig,
}

#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    pub trace: String,
    pub algorithms: Vec<String>,
}

impl IntoResponse for AbrError {
    fn into_response(self) -> Response {
        let status = match self {
            AbrError::UnknownTrace(_) => StatusCode::NOT_FOUND,
            AbrError::UnknownAlgorithm(_) | AbrError::NoAlgorithms => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(serde_json::json!({ "error": self.to_string() }))).into_response()
    }
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/traces", get(traces))
        .route("/api/algorithms", get(algorithms))
        .route("/api/config", get(config))
        .route("/api/simulate", post(simulate))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn traces(State(state): State<Arc<AppState>>) -> Json<Vec<TraceSummary>> {
    Json(state.traces.iter().map(Trace::summary).collect())
}

async fn algorithms() -> Json<Vec<AlgorithmInfo>> {
    Json(catalog())
}

async fn config(State(state): State<Arc<AppState>>) -> Json<PlayerConfig> {
    Json(state.config.clone())
}

async fn simulate(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SimulateRequest>,
) -> Result<Json<SimulationReport>, AbrError> {
    let trace = state
        .traces
        .iter()
        .find(|t| t.id == request.trace)
        .ok_or_else(|| AbrError::UnknownTrace(request.trace.clone()))?;
    Ok(Json(run(&state.config, trace, &request.algorithms)?))
}
