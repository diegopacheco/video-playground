mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::fake_pipeline;
use http_body_util::BodyExt;
use per_title::api::{AppState, router};
use per_title::store::MemoryStore;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

async fn call(state: &AppState, method: &str, uri: &str) -> (StatusCode, Value) {
    let response = router(state.clone())
        .oneshot(Request::builder().method(method).uri(uri).body(Body::empty()).expect("request"))
        .await
        .expect("response");
    let status = response.status();
    let bytes = response.into_body().collect().await.expect("body").to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn analysis_is_404_until_a_run_completes_then_served_from_cache() {
    let state = AppState::new(fake_pipeline("api", false), Arc::new(MemoryStore::default()));
    let (status, _) = call(&state, "GET", "/api/analysis").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, body) = call(&state, "POST", "/api/analysis").await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["total"], 12);

    let mut finished = false;
    for _ in 0..100 {
        let (_, s) = call(&state, "GET", "/api/status").await;
        if s["state"] == "done" {
            finished = true;
            break;
        }
        assert_ne!(s["state"], "failed", "{s}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(finished);

    let (status, body) = call(&state, "GET", "/api/analysis").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["titles"].as_array().map(|a| a.len()), Some(2));
}

#[tokio::test]
async fn a_second_run_is_refused_while_one_is_in_flight() {
    let state = AppState::new(fake_pipeline("busy", false), Arc::new(MemoryStore::default()));
    if let Ok(mut s) = state.status.lock() {
        s.state = per_title::api::RunState::Running;
    }
    let (status, _) = call(&state, "POST", "/api/analysis").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn failures_surface_in_status_with_the_reason() {
    let state = AppState::new(fake_pipeline("apifail", true), Arc::new(MemoryStore::default()));
    call(&state, "POST", "/api/analysis").await;
    let mut seen = Value::Null;
    for _ in 0..100 {
        let (_, s) = call(&state, "GET", "/api/status").await;
        if s["state"] == "failed" {
            seen = s;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(seen["error"].as_str().unwrap_or("").contains("encoder crashed"));
}

#[tokio::test]
async fn config_exposes_grid_and_generic_ladder_for_the_ui() {
    let state = AppState::new(fake_pipeline("cfg", false), Arc::new(MemoryStore::default()));
    let (status, body) = call(&state, "GET", "/api/config").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["generic_ladder"].as_array().map(|a| a.len()), Some(2));
    assert_eq!(body["bitrates_kbps"].as_array().map(|a| a.len()), Some(3));
}
