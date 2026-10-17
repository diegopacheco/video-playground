use crate::error::AppError;
use crate::service::{Decision, PlayOutcome, Scheduler};
use axum::extract::{ConnectInfo, FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

pub type AppState = Arc<Scheduler>;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(|| async { Json(json!({ "status": "ok" })) }))
        .route("/api/clock", get(clock).post(set_clock))
        .route("/api/board", get(board))
        .route("/api/episodes", get(episodes))
        .route("/api/episodes/{id}/entitlement", get(entitlement))
        .route("/api/episodes/{id}/play", post(play))
        .route("/api/episodes/{id}/release", post(release))
        .route("/api/metrics", get(metrics))
        .route("/api/metrics/reset", post(reset_metrics))
        .with_state(state)
}

pub struct ClientGeo {
    pub header: Option<String>,
    pub ip: Option<IpAddr>,
}

impl<S: Send + Sync> FromRequestParts<S> for ClientGeo {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let header_str = |name: &str| parts.headers.get(name).and_then(|v| v.to_str().ok());
        let forwarded = header_str("x-forwarded-for")
            .and_then(|v| v.split(',').next())
            .and_then(|v| v.trim().parse::<IpAddr>().ok());
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip());
        Ok(ClientGeo {
            header: header_str("x-country").map(str::to_string),
            ip: forwarded.or(peer),
        })
    }
}

#[derive(Deserialize)]
struct OffsetBody {
    offset_ms: i64,
}

#[derive(Deserialize)]
struct ReleaseBody {
    in_secs: i64,
}

fn clock_json(s: &Scheduler) -> Json<serde_json::Value> {
    Json(json!({ "server_now": s.now(), "offset_ms": s.offset_ms() }))
}

async fn clock(State(s): State<AppState>) -> Json<serde_json::Value> {
    clock_json(&s)
}

async fn set_clock(State(s): State<AppState>, Json(body): Json<OffsetBody>) -> Json<serde_json::Value> {
    s.set_offset_ms(body.offset_ms);
    clock_json(&s)
}

async fn board(State(s): State<AppState>) -> Result<Response, AppError> {
    Ok(Json(s.board()?).into_response())
}

async fn episodes(State(s): State<AppState>) -> Result<Response, AppError> {
    Ok(Json(s.episodes()?).into_response())
}

async fn release(State(s): State<AppState>, Path(id): Path<String>, Json(body): Json<ReleaseBody>) -> Result<Response, AppError> {
    Ok(Json(s.reschedule(&id, body.in_secs)?).into_response())
}

async fn metrics(State(s): State<AppState>) -> Response {
    Json(s.metrics()).into_response()
}

async fn reset_metrics(State(s): State<AppState>) -> Response {
    s.reset_metrics();
    Json(s.metrics()).into_response()
}

fn with_retry_after(mut response: Response, retry_after_ms: u64) -> Response {
    let secs = retry_after_ms.div_ceil(1000).max(1);
    let headers: &mut HeaderMap = response.headers_mut();
    headers.insert("retry-after", HeaderValue::from(secs));
    headers.insert("x-retry-after-ms", HeaderValue::from(retry_after_ms));
    response
}

fn throttled(retry_after_ms: u64) -> Response {
    let body = Json(json!({ "status": "throttled", "retry_after_ms": retry_after_ms }));
    with_retry_after((StatusCode::TOO_MANY_REQUESTS, body).into_response(), retry_after_ms)
}

async fn entitlement(State(s): State<AppState>, Path(id): Path<String>, geo: ClientGeo) -> Result<Response, AppError> {
    let country = s.resolve_country(geo.header.as_deref(), geo.ip);
    match s.entitlement(&id, &country)? {
        Decision::Throttled { retry_after_ms } => Ok(throttled(retry_after_ms)),
        Decision::Served { view, cache_hit, retry_after_ms } => {
            let mut response = Json(view).into_response();
            let cache = HeaderValue::from_static(if cache_hit { "HIT" } else { "MISS" });
            response.headers_mut().insert("x-cache", cache);
            Ok(match retry_after_ms {
                Some(ms) => with_retry_after(response, ms),
                None => response,
            })
        }
    }
}

async fn play(State(s): State<AppState>, Path(id): Path<String>, geo: ClientGeo) -> Result<Response, AppError> {
    let country = s.resolve_country(geo.header.as_deref(), geo.ip);
    Ok(match s.play(&id, &country)? {
        PlayOutcome::Granted { session_id, view } => {
            Json(json!({ "session_id": session_id, "entitlement": view })).into_response()
        }
        PlayOutcome::Locked { retry_after_ms, view } => {
            with_retry_after((StatusCode::TOO_EARLY, Json(view)).into_response(), retry_after_ms)
        }
        PlayOutcome::Blocked { view } => {
            (StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS, Json(view)).into_response()
        }
        PlayOutcome::Throttled { retry_after_ms } => throttled(retry_after_ms),
    })
}
