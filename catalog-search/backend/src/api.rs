use std::sync::Arc;

use axum::extract::{Query as Params, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::engine::DefaultEngine;
use crate::error::SearchError;
use crate::index::IndexStats;

const MAX_LIMIT: usize = 50;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct Suggestion {
    pub id: u32,
    pub title: String,
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

impl IntoResponse for SearchError {
    fn into_response(self) -> Response {
        let status = match self {
            SearchError::EmptyQuery => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(ApiError { error: self.to_string() })).into_response()
    }
}

type Shared = Arc<DefaultEngine>;

async fn search(State(engine): State<Shared>, Params(p): Params<SearchParams>) -> Result<Response, SearchError> {
    let limit = p.limit.unwrap_or(10).clamp(1, MAX_LIMIT);
    Ok(Json(engine.search(&p.q, limit)?).into_response())
}

async fn suggest(State(engine): State<Shared>, Params(p): Params<SearchParams>) -> Result<Json<Vec<Suggestion>>, SearchError> {
    let limit = p.limit.unwrap_or(6).clamp(1, MAX_LIMIT);
    let response = engine.search(&p.q, limit)?;
    let items = response
        .results
        .into_iter()
        .map(|h| {
            let title = h
                .highlights
                .iter()
                .find(|f| f.field == h.breakdown.field)
                .map(|f| f.segments.iter().map(|s| s.text.as_str()).collect())
                .unwrap_or(h.romaji);
            Suggestion { id: h.id, title }
        })
        .collect();
    Ok(Json(items))
}

async fn stats(State(engine): State<Shared>) -> Json<IndexStats> {
    Json(engine.stats())
}

async fn health() -> &'static str {
    "ok"
}

pub fn router(engine: Shared) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/search", get(search))
        .route("/api/suggest", get(suggest))
        .route("/api/stats", get(stats))
        .with_state(engine)
}
