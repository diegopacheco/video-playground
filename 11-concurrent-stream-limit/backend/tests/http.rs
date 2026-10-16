use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use streamlimit::clock::ManualClock;
use streamlimit::http::router;
use streamlimit::memory::MemoryLeaseStore;
use streamlimit::plan::{Policy, default_accounts};
use streamlimit::service::LeaseService;
use tower::ServiceExt;

fn app() -> (Router, Arc<ManualClock>) {
    let clock = Arc::new(ManualClock::new(5_000));
    let svc = Arc::new(LeaseService::new(
        MemoryLeaseStore::new(),
        clock.clone(),
        10_000,
        default_accounts(),
        Policy::RejectNewest,
    ));
    (router(svc), clock)
}

async fn call(app: &Router, method: &str, uri: &str, body: Option<&str>) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    if body.is_some() {
        req = req.header("content-type", "application/json");
    }
    let req = req
        .body(Body::from(body.unwrap_or("").to_string()))
        .expect("request");
    let res = app.clone().oneshot(req).await.expect("response");
    let status = res.status();
    let bytes = to_bytes(res.into_body(), 1 << 20).await.expect("body");
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

#[tokio::test]
async fn second_screen_on_basic_plan_gets_409_so_the_player_can_show_a_limit_message() {
    let (app, _) = app();
    let (s, b) = call(&app, "POST", "/api/accounts/alice/devices/tv/acquire", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b["status"], "granted");
    let (s, b) = call(&app, "POST", "/api/accounts/alice/devices/phone/acquire", None).await;
    assert_eq!(s, StatusCode::CONFLICT);
    assert_eq!(b["limit"], 1);
}

#[tokio::test]
async fn kicked_device_gets_410_on_heartbeat_so_its_player_stops() {
    let (app, clock) = app();
    let (s, _) = call(&app, "PUT", "/api/policy", Some(r#"{"policy":"kick_oldest"}"#)).await;
    assert_eq!(s, StatusCode::OK);
    call(&app, "POST", "/api/accounts/alice/devices/tv/acquire", None).await;
    clock.advance(1);
    let (_, b) = call(&app, "POST", "/api/accounts/alice/devices/phone/acquire", None).await;
    assert_eq!(b["evicted"][0], "tv");
    let (s, _) = call(&app, "POST", "/api/accounts/alice/devices/tv/heartbeat", None).await;
    assert_eq!(s, StatusCode::GONE);
}

#[tokio::test]
async fn snapshot_reports_server_time_so_the_ui_countdown_is_not_skewed() {
    let (app, _) = app();
    call(&app, "POST", "/api/accounts/carol/devices/tv/acquire", None).await;
    let (s, b) = call(&app, "GET", "/api/accounts/carol/leases", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b["now_ms"], 5_000);
    assert_eq!(b["account"]["limit"], 4);
    assert_eq!(b["leases"][0]["expires_at_ms"], 15_000);
}

#[tokio::test]
async fn bad_inputs_map_to_client_errors() {
    let (app, _) = app();
    let (s, _) = call(&app, "GET", "/api/accounts/nobody/leases", None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (s, _) = call(&app, "PUT", "/api/policy", Some(r#"{"policy":"random"}"#)).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}
