use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use tower_http::services::ServeDir;

use crate::catalog::{Catalog, DEFAULT_LOCALE, Track};
use crate::error::SubError;
use crate::hls::{Rendition, master_playlist, playlist_duration, subtitle_playlist, with_timestamp_map};

#[derive(Clone)]
pub struct AppState {
    pub catalog: Arc<Catalog>,
    pub media_dir: PathBuf,
}

impl IntoResponse for SubError {
    fn into_response(self) -> Response {
        let status = match self {
            SubError::UnknownLocale(_) => StatusCode::NOT_FOUND,
            SubError::MediaMissing(_) => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

pub fn router(state: AppState) -> Router {
    let media = ServeDir::new(state.media_dir.clone());
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/tracks", get(list_tracks))
        .route("/api/tracks/{locale}/ass", get(track_ass))
        .route("/api/tracks/{locale}/vtt", get(track_vtt))
        .route("/api/tracks/{locale}/report", get(track_report))
        .route("/hls/master.m3u8", get(hls_master))
        .route("/hls/subs/{locale}/index.m3u8", get(hls_sub_playlist))
        .route("/hls/subs/{locale}/subs.vtt", get(hls_sub_vtt))
        .nest_service("/media", media)
        .with_state(state)
}

fn text(content_type: &'static str, body: String) -> Response {
    ([(header::CONTENT_TYPE, content_type)], body).into_response()
}

async fn list_tracks(State(state): State<AppState>) -> Json<Vec<Track>> {
    Json(state.catalog.tracks().to_vec())
}

async fn track_ass(State(state): State<AppState>, Path(locale): Path<String>) -> Result<Response, SubError> {
    let track = state.catalog.get(&locale)?;
    Ok(text("text/x-ssa; charset=utf-8", track.ass.clone()))
}

async fn track_vtt(State(state): State<AppState>, Path(locale): Path<String>) -> Result<Response, SubError> {
    let track = state.catalog.get(&locale)?;
    Ok(text("text/vtt; charset=utf-8", track.vtt.clone()))
}

async fn track_report(State(state): State<AppState>, Path(locale): Path<String>) -> Result<Response, SubError> {
    let track = state.catalog.get(&locale)?;
    Ok(Json(track.report.clone()).into_response())
}

const HLS_TYPE: &str = "application/vnd.apple.mpegurl";

fn read_media(state: &AppState, rel: &str) -> Result<String, SubError> {
    std::fs::read_to_string(state.media_dir.join(rel))
        .map_err(|_| SubError::MediaMissing(format!("{rel} not found, run scripts/setup.sh")))
}

async fn hls_master(State(state): State<AppState>) -> Result<Response, SubError> {
    let video_master = read_media(&state, "hls/master.m3u8")?;
    let renditions: Vec<Rendition> = state
        .catalog
        .tracks()
        .iter()
        .map(|t| Rendition {
            locale: t.locale.clone(),
            name: t.name.clone(),
            default: t.locale == DEFAULT_LOCALE,
        })
        .collect();
    let body = master_playlist(&video_master, "/media/hls/", &renditions)?;
    Ok(text(HLS_TYPE, body))
}

async fn hls_sub_playlist(State(state): State<AppState>, Path(locale): Path<String>) -> Result<Response, SubError> {
    state.catalog.get(&locale)?;
    let duration = playlist_duration(&read_media(&state, "hls/video.m3u8")?)?;
    Ok(text(HLS_TYPE, subtitle_playlist(duration)))
}

async fn hls_sub_vtt(State(state): State<AppState>, Path(locale): Path<String>) -> Result<Response, SubError> {
    let track = state.catalog.get(&locale)?;
    Ok(text("text/vtt; charset=utf-8", with_timestamp_map(&track.vtt)))
}
