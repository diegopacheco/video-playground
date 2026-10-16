use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use subtitle_service::ass::AssParser;
use subtitle_service::catalog::{Catalog, FsTrackSource};
use subtitle_service::http::{AppState, router};
use tower::ServiceExt;

fn app() -> axum::Router {
    let subs = concat!(env!("CARGO_MANIFEST_DIR"), "/../subs");
    let catalog = Catalog::build(&FsTrackSource::new(subs), &AssParser).unwrap();
    router(AppState {
        catalog: Arc::new(catalog),
        media_dir: std::env::temp_dir().join("vp06-missing-media"),
    })
}

async fn get(path: &str) -> (StatusCode, String, String) {
    let res = app().oneshot(Request::get(path).body(Body::empty()).unwrap()).await.unwrap();
    let status = res.status();
    let content_type = res
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    (status, content_type, String::from_utf8(body.to_vec()).unwrap())
}

#[tokio::test]
async fn every_locale_is_listed_with_bytes_and_a_lossiness_report() {
    let (status, _, body) = get("/api/tracks").await;
    assert_eq!(status, StatusCode::OK);
    let tracks: serde_json::Value = serde_json::from_str(&body).unwrap();
    let locales: Vec<&str> = tracks.as_array().unwrap().iter().map(|t| t["locale"].as_str().unwrap()).collect();
    assert_eq!(locales, ["en-US", "es-419", "ja-JP", "pt-BR"]);
    for t in tracks.as_array().unwrap() {
        assert!(t["report"]["fidelity"].as_f64().unwrap() < 1.0, "committed tracks use effects WebVTT cannot express");
        assert!(t["vttBytes"].as_u64().unwrap() > 0);
    }
}

#[tokio::test]
async fn vtt_is_served_with_the_mime_type_browsers_require_for_track_elements() {
    let (status, content_type, body) = get("/api/tracks/ja-JP/vtt").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.starts_with("text/vtt"));
    assert!(body.starts_with("WEBVTT"));
    assert!(body.contains("字幕ラボ"));
}

#[tokio::test]
async fn raw_ass_is_served_for_client_side_rendering() {
    let (status, _, body) = get("/api/tracks/pt-BR/ass").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("[V4+ Styles]"));
}

#[tokio::test]
async fn unknown_locale_is_404_not_a_silent_fallback() {
    assert_eq!(get("/api/tracks/xx-XX/vtt").await.0, StatusCode::NOT_FOUND);
    assert_eq!(get("/hls/subs/xx-XX/subs.vtt").await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn hls_master_without_generated_media_is_503_with_a_hint() {
    let (status, _, body) = get("/hls/master.m3u8").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body.contains("setup.sh"));
}

#[tokio::test]
async fn hls_subtitle_segment_is_mapped_vtt() {
    let (status, _, body) = get("/hls/subs/es-419/subs.vtt").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.starts_with("WEBVTT\nX-TIMESTAMP-MAP="));
}
