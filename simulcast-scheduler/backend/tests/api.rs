mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Duration;
use common::{release, rig};
use http_body_util::BodyExt;
use simulcast_scheduler::api::router;
use simulcast_scheduler::service::HerdConfig;
use tower::ServiceExt;

async fn call(app: axum::Router, method: &str, uri: &str, country: Option<&str>) -> (StatusCode, axum::http::HeaderMap, serde_json::Value) {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(c) = country {
        req = req.header("x-country", c);
    }
    let res = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, headers, serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null))
}

#[tokio::test]
async fn locked_entitlement_carries_countdown_local_time_and_retry_after() {
    let r = rig(release() - Duration::seconds(5), HerdConfig::default());
    let (status, headers, body) = call(router(r.scheduler.clone()), "GET", "/api/episodes/ep/entitlement", Some("US")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "locked");
    assert_eq!(body["countdown_ms"], 5_000);
    assert_eq!(body["local_release"]["local"], "2026-11-01 01:00:00");
    assert_eq!(body["local_release"]["abbrev"], "EST");
    let retry: u64 = headers["retry-after"].to_str().unwrap().parse().unwrap();
    assert!((5..=8).contains(&retry));
}

#[tokio::test]
async fn play_before_unlock_is_too_early_and_blocked_region_is_451() {
    let r = rig(release() - Duration::seconds(5), HerdConfig::default());
    let (early, headers, _) = call(router(r.scheduler.clone()), "POST", "/api/episodes/ep/play", Some("US")).await;
    assert_eq!(early, StatusCode::TOO_EARLY);
    assert!(headers.contains_key("retry-after"));
    let (blocked, _, body) = call(router(r.scheduler.clone()), "POST", "/api/episodes/ep/play", Some("CN")).await;
    assert_eq!(blocked, StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS);
    assert_eq!(body["reason"], "not_licensed");
}

#[tokio::test]
async fn play_after_unlock_issues_a_session() {
    let r = rig(release(), HerdConfig::default());
    let (status, _, body) = call(router(r.scheduler.clone()), "POST", "/api/episodes/ep/play", Some("GB")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["session_id"].as_str().unwrap().starts_with("ep-GB-"));
}

#[tokio::test]
async fn unknown_episode_is_404() {
    let r = rig(release(), HerdConfig::default());
    let (status, _, _) = call(router(r.scheduler.clone()), "GET", "/api/episodes/nope/entitlement", Some("US")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn throttled_requests_get_429_with_retry_after() {
    let config = HerdConfig { bucket_capacity: 1.0, bucket_refill_per_sec: 0.0, ..HerdConfig::default() };
    let r = rig(release(), config);
    call(router(r.scheduler.clone()), "GET", "/api/episodes/ep/entitlement", Some("US")).await;
    let (status, headers, _) = call(router(r.scheduler.clone()), "GET", "/api/episodes/ep/entitlement", Some("BR")).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    let retry: u64 = headers["retry-after"].to_str().unwrap().parse().unwrap();
    assert!((1..=3).contains(&retry));
}

#[tokio::test]
async fn board_lists_every_region_for_every_episode() {
    let r = rig(release(), HerdConfig::default());
    let (status, _, body) = call(router(r.scheduler.clone()), "GET", "/api/board", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["episodes"][0]["regions"].as_array().unwrap().len(), 9);
}
