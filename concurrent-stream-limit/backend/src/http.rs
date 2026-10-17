use crate::error::LeaseError;
use crate::lease::{AcquireOutcome, LeaseStore};
use crate::plan::Policy;
use crate::service::LeaseService;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

impl IntoResponse for LeaseError {
    fn into_response(self) -> Response {
        let status = match self {
            LeaseError::UnknownAccount(_) => StatusCode::NOT_FOUND,
            LeaseError::InvalidDevice | LeaseError::InvalidPolicy(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

#[derive(Deserialize)]
struct PolicyBody {
    policy: String,
}

type Svc<S> = State<Arc<LeaseService<S>>>;

pub fn router<S: LeaseStore>(svc: Arc<LeaseService<S>>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/accounts", get(accounts::<S>))
        .route("/api/policy", get(get_policy::<S>).put(put_policy::<S>))
        .route("/api/accounts/{account}/leases", get(leases::<S>))
        .route(
            "/api/accounts/{account}/devices/{device}/acquire",
            post(acquire::<S>),
        )
        .route(
            "/api/accounts/{account}/devices/{device}/heartbeat",
            post(heartbeat::<S>),
        )
        .route(
            "/api/accounts/{account}/devices/{device}/release",
            post(release::<S>),
        )
        .with_state(svc)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn accounts<S: LeaseStore>(State(svc): Svc<S>) -> Json<serde_json::Value> {
    Json(json!({ "accounts": svc.accounts(), "ttl_ms": svc.ttl_ms() }))
}

async fn get_policy<S: LeaseStore>(State(svc): Svc<S>) -> Result<Response, LeaseError> {
    Ok(Json(json!({ "policy": svc.policy()? })).into_response())
}

async fn put_policy<S: LeaseStore>(
    State(svc): Svc<S>,
    Json(body): Json<PolicyBody>,
) -> Result<Response, LeaseError> {
    let policy = Policy::parse(&body.policy)?;
    svc.set_policy(policy)?;
    Ok(Json(json!({ "policy": policy })).into_response())
}

async fn leases<S: LeaseStore>(
    State(svc): Svc<S>,
    Path(account): Path<String>,
) -> Result<Response, LeaseError> {
    Ok(Json(svc.snapshot(&account).await?).into_response())
}

async fn acquire<S: LeaseStore>(
    State(svc): Svc<S>,
    Path((account, device)): Path<(String, String)>,
) -> Result<Response, LeaseError> {
    let limit = svc.account(&account)?.limit;
    Ok(match svc.acquire(&account, &device).await? {
        AcquireOutcome::Granted { lease, evicted } => Json(json!({
            "status": "granted",
            "lease": lease,
            "evicted": evicted,
        }))
        .into_response(),
        AcquireOutcome::Rejected { active } => (
            StatusCode::CONFLICT,
            Json(json!({
                "status": "rejected",
                "active": active,
                "limit": limit,
            })),
        )
            .into_response(),
    })
}

async fn heartbeat<S: LeaseStore>(
    State(svc): Svc<S>,
    Path((account, device)): Path<(String, String)>,
) -> Result<Response, LeaseError> {
    Ok(match svc.heartbeat(&account, &device).await? {
        Some(lease) => Json(json!({ "status": "renewed", "lease": lease })).into_response(),
        None => (StatusCode::GONE, Json(json!({ "status": "lost" }))).into_response(),
    })
}

async fn release<S: LeaseStore>(
    State(svc): Svc<S>,
    Path((account, device)): Path<(String, String)>,
) -> Result<Response, LeaseError> {
    let released = svc.release(&account, &device).await?;
    Ok(Json(json!({ "released": released })).into_response())
}
