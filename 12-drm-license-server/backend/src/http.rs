use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::audit::{AuditEvent, AuditLog};
use crate::error::DrmError;
use crate::license::LicenseService;
use crate::session::{Claims, SessionSigner};
use crate::url_signer::UrlSigner;

pub const SESSION_TTL: u64 = 600;
pub const URL_TTL: u64 = 300;

pub struct AppState {
    pub sessions: SessionSigner,
    pub urls: UrlSigner,
    pub licenses: HashMap<String, Arc<dyn LicenseService>>,
    pub media_dir: PathBuf,
    pub audit: AuditLog,
}

type Shared = Arc<AppState>;

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/session", post(create_session))
        .route("/api/playback", post(create_playback))
        .route("/api/license/{system}", post(issue_license))
        .route("/api/audit", get(audit))
        .route("/media/{expires}/{sig}/{content}/{file}", get(media))
        .with_state(Arc::new(state))
}

fn audited<T>(state: &AppState, kind: &str, result: Result<T, DrmError>, ok: impl FnOnce(&T) -> String) -> Result<T, DrmError> {
    match &result {
        Ok(value) => state.audit.record(now(), kind, "granted", ok(value)),
        Err(e) => state.audit.record(now(), kind, "denied", e.to_string()),
    }
    result
}

fn bearer(headers: &HeaderMap) -> Result<&str, DrmError> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(DrmError::Unauthenticated)
}

fn authenticate(state: &AppState, headers: &HeaderMap) -> Result<Claims, DrmError> {
    state.sessions.verify(bearer(headers)?, now())
}

#[derive(Deserialize)]
pub struct SessionRequest {
    pub user: String,
    pub entitled: bool,
    #[serde(default)]
    pub expired: bool,
}

#[derive(Serialize)]
pub struct SessionResponse {
    pub token: String,
    pub claims: Claims,
}

async fn create_session(State(state): State<Shared>, Json(req): Json<SessionRequest>) -> Result<Json<SessionResponse>, DrmError> {
    let exp = if req.expired { now() - 1 } else { now() + SESSION_TTL };
    let ent = if req.entitled { vec!["clip".to_string()] } else { vec![] };
    let claims = Claims { sub: req.user, ent, exp };
    let result = state.sessions.issue(&claims).map(|token| SessionResponse { token, claims });
    audited(&state, "session", result, |r| format!("{} ent={:?}", r.claims.sub, r.claims.ent)).map(Json)
}

#[derive(Deserialize)]
pub struct PlaybackRequest {
    pub content: String,
    #[serde(default)]
    pub expired: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackResponse {
    pub manifest_url: String,
    pub expires: u64,
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') && !name.starts_with('.')
}

async fn create_playback(State(state): State<Shared>, headers: HeaderMap, Json(req): Json<PlaybackRequest>) -> Result<Json<PlaybackResponse>, DrmError> {
    let result = playback(&state, &headers, &req);
    audited(&state, "playback", result, |r| r.manifest_url.clone()).map(Json)
}

fn playback(state: &AppState, headers: &HeaderMap, req: &PlaybackRequest) -> Result<PlaybackResponse, DrmError> {
    authenticate(state, headers)?;
    if !valid_name(&req.content) || !state.media_dir.join(&req.content).join("stream.mpd").is_file() {
        return Err(DrmError::NotFound);
    }
    let expires = if req.expired { now() - 1 } else { now() + URL_TTL };
    let manifest_url = state.urls.signed_path(&req.content, "stream.mpd", expires)?;
    Ok(PlaybackResponse { manifest_url, expires })
}

async fn issue_license(State(state): State<Shared>, Path(system): Path<String>, headers: HeaderMap, body: axum::body::Bytes) -> Result<Response, DrmError> {
    let result = license(&state, &system, &headers, &body);
    let grant = audited(&state, &format!("license:{system}"), result, |g| format!("keys for {:?}", g.contents))?;
    Ok(([(header::CONTENT_TYPE, HeaderValue::from_static(grant.content_type))], grant.body).into_response())
}

fn license(state: &AppState, system: &str, headers: &HeaderMap, body: &[u8]) -> Result<crate::license::LicenseGrant, DrmError> {
    let service = state.licenses.get(system).ok_or(DrmError::NotFound)?;
    let claims = authenticate(state, headers)?;
    service.issue(body, &claims)
}

async fn audit(State(state): State<Shared>) -> Json<Vec<AuditEvent>> {
    Json(state.audit.recent())
}

fn content_type(file: &str) -> &'static str {
    if file.ends_with(".mpd") {
        "application/dash+xml"
    } else if file.ends_with(".m4s") || file.ends_with(".mp4") {
        "video/mp4"
    } else {
        "application/octet-stream"
    }
}

async fn media(State(state): State<Shared>, Path((expires, sig, content, file)): Path<(u64, String, String, String)>) -> Result<Response, DrmError> {
    let checked = state.urls.verify(&content, expires, &sig, now()).and_then(|_| {
        if valid_name(&content) && valid_name(&file) { Ok(()) } else { Err(DrmError::NotFound) }
    });
    if let Err(e) = checked {
        state.audit.record(now(), "segment", "denied", format!("{content}/{file}: {e}"));
        return Err(e);
    }
    let bytes = tokio::fs::read(state.media_dir.join(&content).join(&file)).await.map_err(|_| DrmError::NotFound)?;
    if file.ends_with(".mpd") {
        state.audit.record(now(), "segment", "granted", format!("{content}/{file}"));
    }
    Ok(([(header::CONTENT_TYPE, content_type(&file))], bytes).into_response())
}
