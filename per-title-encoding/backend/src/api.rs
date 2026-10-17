use crate::error::AppError;
use crate::pipeline::Pipeline;
use crate::store::AnalysisStore;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use serde_json::json;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RunState {
    Idle,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunStatus {
    pub state: RunState,
    pub done: usize,
    pub total: usize,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub pipeline: Arc<Pipeline>,
    pub store: Arc<dyn AnalysisStore>,
    pub status: Arc<Mutex<RunStatus>>,
}

impl AppState {
    pub fn new(pipeline: Pipeline, store: Arc<dyn AnalysisStore>) -> Self {
        Self {
            pipeline: Arc::new(pipeline),
            store,
            status: Arc::new(Mutex::new(RunStatus {
                state: RunState::Idle,
                done: 0,
                total: 0,
                error: None,
            })),
        }
    }

    fn update(&self, f: impl FnOnce(&mut RunStatus)) {
        if let Ok(mut s) = self.status.lock() {
            f(&mut s);
        }
    }

    pub fn snapshot(&self) -> RunStatus {
        self.status.lock().map(|s| s.clone()).unwrap_or(RunStatus {
            state: RunState::Failed,
            done: 0,
            total: 0,
            error: Some("status lock poisoned".into()),
        })
    }

    pub fn start(&self) -> Result<RunStatus, AppError> {
        {
            let mut s = self.status.lock().map_err(|_| AppError::Busy)?;
            if s.state == RunState::Running {
                return Err(AppError::Busy);
            }
            *s = RunStatus {
                state: RunState::Running,
                done: 0,
                total: self.pipeline.total_jobs(),
                error: None,
            };
        }
        let state = self.clone();
        tokio::task::spawn_blocking(move || {
            let progress = |done: usize, total: usize| {
                state.update(|s| {
                    s.done = done;
                    s.total = total;
                })
            };
            let outcome = state
                .pipeline
                .run(&progress)
                .and_then(|analysis| state.store.save(&analysis));
            state.update(|s| match outcome {
                Ok(()) => s.state = RunState::Done,
                Err(e) => {
                    s.state = RunState::Failed;
                    s.error = Some(e.to_string());
                }
            });
        });
        Ok(self.snapshot())
    }
}

fn error(status: StatusCode, message: String) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn config(State(state): State<AppState>) -> Response {
    Json(&state.pipeline.config).into_response()
}

async fn analysis(State(state): State<AppState>) -> Response {
    match state.store.load() {
        Ok(Some(a)) => Json(a).into_response(),
        Ok(None) => error(StatusCode::NOT_FOUND, "no analysis cached yet, POST /api/analysis".into()),
        Err(e) => error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn start(State(state): State<AppState>) -> Response {
    match state.start() {
        Ok(s) => (StatusCode::ACCEPTED, Json(s)).into_response(),
        Err(e) => error(StatusCode::CONFLICT, e.to_string()),
    }
}

async fn status(State(state): State<AppState>) -> Json<RunStatus> {
    Json(state.snapshot())
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/config", get(config))
        .route("/api/analysis", get(analysis).post(start))
        .route("/api/status", get(status))
        .with_state(state)
}
