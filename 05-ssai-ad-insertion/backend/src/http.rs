use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::ads::{Catalog, Creative};
use crate::error::{Result, SsaiError};
use crate::playlist::Playlist;
use crate::stitcher::{ManifestStitcher, StitchedBreak};
use crate::tracking::{Beacon, BeaconSink, events_for_segment};
use crate::users::{UserProfile, all_users, find_user};

pub struct AppState {
    pub raw: String,
    pub content: Playlist,
    pub stitcher: ManifestStitcher,
    pub catalog: Arc<Catalog>,
    pub beacons: Arc<dyn BeaconSink>,
    pub media_dir: PathBuf,
}

#[derive(Deserialize)]
pub struct StitchQuery {
    user: String,
    session: Option<String>,
}

#[derive(Deserialize)]
pub struct BeaconQuery {
    session: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StitchView {
    user: UserProfile,
    session: String,
    raw: String,
    stitched: String,
    breaks: Vec<StitchedBreak>,
}

impl IntoResponse for SsaiError {
    fn into_response(self) -> Response {
        let status = match self {
            SsaiError::UnknownUser(_) | SsaiError::UnknownCreative(_) | SsaiError::NotFound(_) => StatusCode::NOT_FOUND,
            SsaiError::InvalidPlaylist(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/users", get(users))
        .route("/api/catalog", get(catalog))
        .route("/api/stitch", get(stitch_view))
        .route("/api/manifest.m3u8", get(manifest))
        .route("/api/track/{user}/{session}/{brk}/{creative}/{file}", get(track))
        .route("/api/beacons", get(beacons).delete(clear_beacons))
        .route("/media/{*path}", get(media))
        .with_state(state)
}

async fn users() -> Json<Vec<UserProfile>> {
    Json(all_users())
}

async fn catalog(State(state): State<Arc<AppState>>) -> Json<Vec<Creative>> {
    Json(state.catalog.creatives.clone())
}

fn session_for(query: &StitchQuery) -> Result<String> {
    let session = query.session.clone().unwrap_or_else(|| query.user.clone());
    let valid = !session.is_empty() && session.len() <= 64 && session.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if valid { Ok(session) } else { Err(SsaiError::NotFound(format!("invalid session {session}"))) }
}

async fn stitch_view(State(state): State<Arc<AppState>>, Query(query): Query<StitchQuery>) -> Result<Json<StitchView>> {
    let user = find_user(&query.user)?;
    let session = session_for(&query)?;
    let stitched = state.stitcher.stitch(&state.content, &user, &session)?;
    Ok(Json(StitchView { user, session, raw: state.raw.clone(), stitched: stitched.playlist.render(), breaks: stitched.breaks }))
}

async fn manifest(State(state): State<Arc<AppState>>, Query(query): Query<StitchQuery>) -> Result<Response> {
    let user = find_user(&query.user)?;
    let session = session_for(&query)?;
    let stitched = state.stitcher.stitch(&state.content, &user, &session)?;
    Ok(([(header::CONTENT_TYPE, "application/vnd.apple.mpegurl"), (header::CACHE_CONTROL, "no-store")], stitched.playlist.render()).into_response())
}

async fn track(
    State(state): State<Arc<AppState>>,
    UrlPath((user, session, brk, creative_id, file)): UrlPath<(String, String, String, String, String)>,
) -> Result<Response> {
    let creative = state.catalog.find(&creative_id)?;
    let index: usize = file
        .strip_suffix(".ts")
        .and_then(|i| i.parse().ok())
        .ok_or_else(|| SsaiError::NotFound(file.clone()))?;
    let segment = creative.segments.get(index).ok_or_else(|| SsaiError::NotFound(file.clone()))?;
    let durations: Vec<f64> = creative.segments.iter().map(|s| s.duration).collect();
    let at_ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    for event in events_for_segment(&durations, index) {
        state.beacons.record(Beacon {
            seq: 0,
            user: user.clone(),
            session: session.clone(),
            break_id: brk.clone(),
            creative: creative.id.clone(),
            event,
            at_ms,
        })?;
    }
    let path = state.media_dir.join("ads").join(&creative.id).join(&segment.file);
    serve_file(&path).await
}

async fn beacons(State(state): State<Arc<AppState>>, Query(query): Query<BeaconQuery>) -> Result<Json<Vec<Beacon>>> {
    Ok(Json(state.beacons.list(query.session.as_deref())?))
}

async fn clear_beacons(State(state): State<Arc<AppState>>) -> Result<StatusCode> {
    state.beacons.clear()?;
    Ok(StatusCode::NO_CONTENT)
}

async fn media(State(state): State<Arc<AppState>>, UrlPath(path): UrlPath<String>) -> Result<Response> {
    let relative = Path::new(&path);
    if !relative.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err(SsaiError::NotFound(path));
    }
    serve_file(&state.media_dir.join(relative)).await
}

async fn serve_file(path: &Path) -> Result<Response> {
    let bytes = tokio::fs::read(path).await.map_err(|_| SsaiError::NotFound(path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()))?;
    let content_type = match path.extension().and_then(|e| e.to_str()) {
        Some("ts") => "video/mp2t",
        Some("m3u8") => "application/vnd.apple.mpegurl",
        _ => "application/octet-stream",
    };
    Ok(([(header::CONTENT_TYPE, content_type)], bytes).into_response())
}
