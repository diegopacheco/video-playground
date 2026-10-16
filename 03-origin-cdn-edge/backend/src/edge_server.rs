use crate::cache::Cache;
use crate::edge::Edge;
use crate::error::CdnError;
use crate::metrics::MetricsSnapshot;
use crate::origin_client::OriginClient;
use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::Instant;

impl IntoResponse for CdnError {
    fn into_response(self) -> Response {
        let status = match self {
            CdnError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_GATEWAY,
        };
        (status, self.to_string()).into_response()
    }
}

fn header_value(value: &str) -> HeaderValue {
    HeaderValue::from_str(value).unwrap_or_else(|_| HeaderValue::from_static("invalid"))
}

async fn segment<C: Cache, O: OriginClient>(
    State(edge): State<Arc<Edge<C, O>>>,
    Path(key): Path<String>,
) -> Result<Response, CdnError> {
    let served = edge.get(&key).await?;
    let mut response = Response::new(Body::from(served.entry.body.clone()));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, header_value(&served.entry.content_type));
    headers.insert(header::CACHE_CONTROL, header_value(&served.entry.policy.header_value()));
    headers.insert(header::AGE, header_value(&served.age.as_secs().to_string()));
    headers.insert("x-cache", HeaderValue::from_static(served.status.as_str()));
    if served.coalesced {
        headers.insert("x-coalesced", HeaderValue::from_static("1"));
    }
    Ok(response)
}

#[derive(Deserialize)]
struct ProbeQuery {
    n: Option<usize>,
}

#[derive(Serialize)]
struct ProbeItem {
    status: &'static str,
    coalesced: bool,
    millis: f64,
}

#[derive(Serialize)]
struct ProbeReport {
    key: String,
    results: Vec<ProbeItem>,
}

async fn probe<C: Cache, O: OriginClient>(
    State(edge): State<Arc<Edge<C, O>>>,
    Path(key): Path<String>,
    Query(query): Query<ProbeQuery>,
) -> Json<ProbeReport> {
    let n = query.n.unwrap_or(50).clamp(1, 1000);
    let tasks: Vec<_> = (0..n)
        .map(|_| {
            let edge = Arc::clone(&edge);
            let key = key.clone();
            tokio::spawn(async move {
                let started = Instant::now();
                let outcome = edge.get(&key).await;
                let millis = started.elapsed().as_secs_f64() * 1000.0;
                match outcome {
                    Ok(served) => ProbeItem { status: served.status.as_str(), coalesced: served.coalesced, millis },
                    Err(_) => ProbeItem { status: "ERROR", coalesced: false, millis },
                }
            })
        })
        .collect();
    let mut results = Vec::with_capacity(n);
    for task in tasks {
        results.push(task.await.unwrap_or(ProbeItem { status: "ERROR", coalesced: false, millis: 0.0 }));
    }
    Json(ProbeReport { key, results })
}

async fn metrics<C: Cache, O: OriginClient>(State(edge): State<Arc<Edge<C, O>>>) -> Json<MetricsSnapshot> {
    Json(edge.metrics())
}

async fn reset<C: Cache, O: OriginClient>(State(edge): State<Arc<Edge<C, O>>>) -> Json<MetricsSnapshot> {
    edge.reset_metrics();
    Json(edge.metrics())
}

pub fn router<C: Cache, O: OriginClient>(edge: Arc<Edge<C, O>>) -> Router {
    Router::new()
        .route("/seg/{key}", get(segment::<C, O>))
        .route("/probe/{key}", post(probe::<C, O>))
        .route("/metrics", get(metrics::<C, O>))
        .route("/metrics/reset", post(reset::<C, O>))
        .route("/health", get(|| async { "ok" }))
        .with_state(edge)
}
