use abr::api::{AppState, router};
use abr::player::PlayerConfig;
use abr::trace::bundled;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

fn app() -> axum::Router {
    router(Arc::new(AppState {
        traces: bundled().unwrap(),
        config: PlayerConfig::default(),
    }))
}

async fn post(body: &str) -> (StatusCode, serde_json::Value) {
    let request = Request::post("/api/simulate")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn simulate_returns_one_run_per_requested_algorithm_in_order() {
    let (status, json) = post(r#"{"trace":"4g-variable","algorithms":["dynamic","throughput"]}"#).await;
    assert_eq!(status, StatusCode::OK);
    let runs = json["runs"].as_array().unwrap();
    assert_eq!(runs[0]["algorithm"], "dynamic");
    assert_eq!(runs[1]["algorithm"], "throughput");
    assert_eq!(runs[0]["session"]["segments"].as_array().unwrap().len(), 150);
    assert_eq!(json["trace"]["bandwidth_kbps"].as_array().unwrap().len(), 600);
}

#[tokio::test]
async fn unknown_algorithm_is_a_client_error() {
    let (status, json) = post(r#"{"trace":"3g","algorithms":["mpc"]}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["error"], "unknown algorithm mpc");
}

#[tokio::test]
async fn unknown_trace_is_not_found() {
    let (status, _) = post(r#"{"trace":"5g","algorithms":["bola"]}"#).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn empty_algorithm_list_is_rejected() {
    let (status, _) = post(r#"{"trace":"3g","algorithms":[]}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn lists_bundled_traces_and_algorithms() {
    let response = app().oneshot(Request::get("/api/algorithms").body(Body::empty()).unwrap()).await.unwrap();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 3);
    let response = app().oneshot(Request::get("/api/traces").body(Body::empty()).unwrap()).await.unwrap();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 4);
}
