use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use catalog_search::api::router;
use catalog_search::catalog::default_catalog;
use catalog_search::engine::DefaultEngine;
use serde_json::Value;
use tower::ServiceExt;

async fn get(uri: &str) -> (StatusCode, Value) {
    let engine = Arc::new(DefaultEngine::with_defaults(default_catalog().expect("catalog")));
    let response = router(engine)
        .oneshot(Request::get(uri).body(Body::empty()).expect("request"))
        .await
        .expect("response");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.expect("body");
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn search_returns_score_breakdown_and_highlight_segments_for_the_ui() {
    let (status, body) = get("/api/search?q=%E9%80%B2%E6%92%83&limit=3").await;
    assert_eq!(status, StatusCode::OK);
    let first = &body["results"][0];
    assert_eq!(first["english"], "Attack on Titan");
    assert_eq!(first["breakdown"]["field"], "japanese");
    assert!(first["breakdown"]["total"].as_f64().unwrap() > 0.0);
    assert!(first["highlights"].as_array().unwrap().len() == 3);
    assert!(body["took_us"].is_u64());
}

#[tokio::test]
async fn blank_query_is_a_client_error() {
    let (status, body) = get("/api/search?q=%20").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("empty"));
}

#[tokio::test]
async fn suggest_returns_short_titles_for_autocomplete() {
    let (status, body) = get("/api/suggest?q=kimet").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["title"], "Kimetsu no Yaiba");
}

#[tokio::test]
async fn stats_reports_the_indexed_catalog_size() {
    let (status, body) = get("/api/stats").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["documents"].as_u64().unwrap() >= 200);
}
