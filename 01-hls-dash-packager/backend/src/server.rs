use crate::error::AppError;
use crate::manifest::{Manifest, SegmentInfo};
use crate::mime;
use axum::extract::{Path as UrlPath, Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub out_dir: PathBuf,
    pub manifest: Arc<Manifest>,
}

#[derive(Serialize)]
pub struct RenditionSummary {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub target_kbps: u32,
    pub average_kbps: f64,
    pub peak_kbps: f64,
    pub codecs: String,
    pub segment_count: usize,
    pub total_bytes: u64,
    pub init_bytes: u64,
    pub playlist: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InvalidPath(_) => StatusCode::BAD_REQUEST,
            AppError::Io(e) if e.kind() == std::io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(serde_json::json!({ "error": self.to_string() }))).into_response()
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/manifest", get(manifest))
        .route("/api/renditions", get(renditions))
        .route("/api/renditions/{id}/segments", get(segments))
        .route("/media/{*path}", get(media))
        .layer(middleware::from_fn(cors))
        .with_state(state)
}

async fn cors(request: Request, next: Next) -> Response {
    let mut response = if request.method() == Method::OPTIONS {
        StatusCode::NO_CONTENT.into_response()
    } else {
        next.run(request).await
    };
    let headers = response.headers_mut();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("GET, OPTIONS"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("Range, Content-Type"));
    headers.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, HeaderValue::from_static("Content-Length, Content-Type"));
    response
}

async fn manifest(State(state): State<AppState>) -> Json<Manifest> {
    Json(state.manifest.as_ref().clone())
}

async fn renditions(State(state): State<AppState>) -> Json<Vec<RenditionSummary>> {
    Json(
        state
            .manifest
            .renditions
            .iter()
            .map(|r| RenditionSummary {
                id: r.id.clone(),
                width: r.width,
                height: r.height,
                target_kbps: r.target_kbps,
                average_kbps: r.average_kbps,
                peak_kbps: r.peak_kbps,
                codecs: r.codecs.clone(),
                segment_count: r.segments.len(),
                total_bytes: r.total_bytes,
                init_bytes: r.init_bytes,
                playlist: r.playlist.clone(),
            })
            .collect(),
    )
}

async fn segments(State(state): State<AppState>, UrlPath(id): UrlPath<String>) -> Result<Json<Vec<SegmentInfo>>, AppError> {
    state
        .manifest
        .renditions
        .iter()
        .find(|r| r.id == id)
        .map(|r| Json(r.segments.clone()))
        .ok_or(AppError::NotFound(format!("rendition {id}")))
}

async fn media(State(state): State<AppState>, UrlPath(path): UrlPath<String>) -> Result<Response, AppError> {
    let file = mime::safe_join(&state.out_dir, &path)?;
    let bytes = tokio::fs::read(&file).await?;
    let cache = if path.ends_with(".m3u8") || path.ends_with(".mpd") {
        "no-cache"
    } else {
        "public, max-age=3600"
    };
    Ok((
        [
            (header::CONTENT_TYPE, mime::content_type(&file)),
            (header::CACHE_CONTROL, cache),
        ],
        bytes,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alignment::AlignmentReport;
    use crate::ladder::Rendition;
    use crate::manifest::{RenditionReport, SegmentFile};
    use axum::body::{Body, to_bytes};
    use tower::ServiceExt;

    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    fn state() -> AppState {
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("vp01-server-{}-{n}", std::process::id()));
        std::fs::create_dir_all(dir.join("480p")).unwrap();
        std::fs::write(dir.join("master.m3u8"), "#EXTM3U\n").unwrap();
        std::fs::write(dir.join("480p/seg_00001.m4s"), [0u8; 16]).unwrap();
        let files = vec![SegmentFile { uri: "seg_00001.m4s".into(), duration: 2.0, bytes: 16 }];
        let report = RenditionReport::build(&Rendition::new("480p", 854, 480, 1000, 30), 8, files, vec![0.0]);
        let manifest = Manifest {
            fps: 24,
            gop_frames: 48,
            gop_seconds: 2.0,
            segment_seconds: 2,
            duration: 2.0,
            hls_master: "master.m3u8".into(),
            dash_mpd: "manifest.mpd".into(),
            renditions: vec![report],
            alignment: AlignmentReport { aligned: true, boundaries_on_keyframes: true, keyframe_times: vec![0.0], issues: vec![] },
        };
        AppState { out_dir: dir, manifest: Arc::new(manifest) }
    }

    async fn get(uri: &str) -> Response {
        router(state()).oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap()).await.unwrap()
    }

    #[tokio::test]
    async fn playlist_is_served_with_hls_mime_and_cors_so_a_player_on_another_origin_can_load_it() {
        let res = get("/media/master.m3u8").await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()[header::CONTENT_TYPE], "application/vnd.apple.mpegurl");
        assert_eq!(res.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
    }

    #[tokio::test]
    async fn segment_is_served_as_iso_segment() {
        let res = get("/media/480p/seg_00001.m4s").await;
        assert_eq!(res.headers()[header::CONTENT_TYPE], "video/iso.segment");
        assert_eq!(to_bytes(res.into_body(), 1024).await.unwrap().len(), 16);
    }

    #[tokio::test]
    async fn preflight_is_answered_without_hitting_routes() {
        let res = router(state())
            .oneshot(Request::builder().method(Method::OPTIONS).uri("/media/master.m3u8").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        assert_eq!(res.headers()[header::ACCESS_CONTROL_ALLOW_HEADERS], "Range, Content-Type");
    }

    #[tokio::test]
    async fn missing_media_and_unknown_rendition_are_404() {
        assert_eq!(get("/media/nope.m4s").await.status(), StatusCode::NOT_FOUND);
        assert_eq!(get("/api/renditions/9000p/segments").await.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn renditions_api_lists_ladder_with_segment_counts() {
        let res = get("/api/renditions").await;
        let body: serde_json::Value = serde_json::from_slice(&to_bytes(res.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body[0]["id"], "480p");
        assert_eq!(body[0]["segment_count"], 1);
        assert_eq!(body[0]["codecs"], "avc1.64001e");
    }
}
