use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use herd::http::{AppState, router};

async fn call(app: &axum::Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    let req = builder.body(body.map(|b| Body::from(b.to_string())).unwrap_or_else(Body::empty)).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn second_joiner_waits_and_cannot_play_until_admitted() {
    let app = router(AppState::new(1).unwrap());
    let (_, first) = call(&app, "POST", "/room/join", None).await;
    let (_, second) = call(&app, "POST", "/room/join", None).await;
    assert_eq!(first["position"], 0);
    assert_eq!(second["position"], 1);
    assert!(second["eta_ms"].as_u64().unwrap() > 900);
    let (_, status) = call(&app, "GET", "/room/status/1", None).await;
    assert_eq!(status["admitted"], false);
    assert!(status["token"].is_null());
    let (code, _) = call(&app, "GET", "/room/play?token=deadbeef", None).await;
    assert_eq!(code, StatusCode::FORBIDDEN);
    let (_, admitted) = call(&app, "GET", "/room/status/0", None).await;
    let token = admitted["token"].as_str().unwrap().to_string();
    let (code, play) = call(&app, "GET", &format!("/room/play?token={token}"), None).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(play["ticket"], 0);
}

#[tokio::test]
async fn unknown_ticket_is_404_and_unknown_strategy_is_400() {
    let app = router(AppState::new(10).unwrap());
    let (code, _) = call(&app, "GET", "/room/status/99", None).await;
    assert_eq!(code, StatusCode::NOT_FOUND);
    let body = serde_json::json!({ "strategy": "yolo", "config": { "users": 10 } });
    let (code, _) = call(&app, "POST", "/api/simulate", Some(body)).await;
    assert_eq!(code, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn compare_returns_every_strategy_with_its_load_curve() {
    let app = router(AppState::new(10).unwrap());
    let body = serde_json::json!({ "config": { "users": 2000, "api_rps": 200, "origin_rps": 50, "admission_rps": 150 } });
    let (code, reports) = call(&app, "POST", "/api/compare", Some(body)).await;
    assert_eq!(code, StatusCode::OK);
    let names: Vec<&str> = reports.as_array().unwrap().iter().map(|r| r["strategy"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["naive", "jittered-backoff", "cdn-prewarm", "waiting-room", "combined"]);
    assert!(reports[0]["series"].as_array().unwrap().len() > 1);
}

#[tokio::test]
async fn reset_empties_the_room() {
    let app = router(AppState::new(10).unwrap());
    call(&app, "POST", "/room/join", None).await;
    let (_, stats) = call(&app, "POST", "/room/reset", None).await;
    assert_eq!(stats["joined"], 0);
    assert_eq!(stats["rate_per_s"], 10);
}
