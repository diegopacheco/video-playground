use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use recs::api::router;
use recs::service::Service;
use recs::store::mem::MemStore;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

async fn app() -> Router {
    router(Arc::new(Service::bootstrap(MemStore::default()).await.expect("bootstrap")))
}

async fn call(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let req = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    let req = req.body(body.map(|b| Body::from(b.to_string())).unwrap_or_else(Body::empty)).expect("request");
    let res = app.clone().oneshot(req).await.expect("response");
    let status = res.status();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

fn row<'a>(recs: &'a Value, kind: &str) -> Option<&'a Value> {
    recs["rows"].as_array().expect("rows").iter().find(|r| r["kind"] == kind)
}

fn ids(row: &Value) -> Vec<i64> {
    row["items"].as_array().expect("items").iter().map(|i| i["title"]["id"].as_i64().expect("id")).collect()
}

#[tokio::test]
async fn new_user_gets_onboarding_driven_rows_then_cf_rows_after_watching() {
    let app = app().await;
    let (status, user) = call(&app, "POST", "/api/users", Some(json!({ "name": "Newcomer", "genres": ["Mecha"] }))).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = user["id"].as_i64().expect("id");

    let (_, recs) = call(&app, "GET", &format!("/api/users/{id}/recommendations"), None).await;
    assert_eq!(recs["cold_start"], true);
    assert!(row(&recs, "because").is_none(), "no watch history means no co-watch row");
    let similar = row(&recs, "similar").expect("similar row from genre picks");
    assert!(similar["label"].as_str().expect("label").contains("Mecha"));
    let top = &row(&recs, "top").expect("top")["items"];
    let mecha = top.as_array().expect("items").iter().filter(|i| i["title"]["genres"].to_string().contains("Mecha")).count();
    assert!(mecha >= 5, "onboarding genre should dominate cold-start picks, got {mecha}");

    let watched = ids(similar)[0];
    let (status, _) = call(&app, "POST", &format!("/api/users/{id}/watches"), Some(json!({ "title_id": watched }))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, recs) = call(&app, "GET", &format!("/api/users/{id}/recommendations?algo=cf"), None).await;
    assert_eq!(recs["cold_start"], false);
    let because = row(&recs, "because").expect("because row after a watch");
    assert_eq!(because["anchor"].as_i64(), Some(watched));
    for r in recs["rows"].as_array().expect("rows") {
        assert!(!ids(r).contains(&watched), "watched title must not be recommended again in {}", r["kind"]);
    }
}

#[tokio::test]
async fn a_new_title_is_served_by_content_only_because_nobody_watched_it() {
    let app = app().await;
    let body = json!({
        "name": "Velvet Thunder Pilots",
        "genres": ["Mecha", "Sci-Fi"],
        "tags": ["giant robots", "space colony"],
        "synopsis": "A reluctant mech pilot defends a dying orbital colony.",
        "year": 2026
    });
    let (status, title) = call(&app, "POST", "/api/titles", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(title["is_new"], true);
    let (_, detail) = call(&app, "GET", &format!("/api/titles/{}", title["id"]), None).await;
    assert!(detail["also_watched"].as_array().expect("cf").is_empty());
    let similar = detail["similar_content"].as_array().expect("content");
    assert!(!similar.is_empty());
    assert!(similar[0]["title"]["genres"].to_string().contains("Mecha") || similar[0]["title"]["genres"].to_string().contains("Sci-Fi"));
}

#[tokio::test]
async fn bad_input_is_rejected_with_clear_status_codes() {
    let app = app().await;
    assert_eq!(call(&app, "GET", "/api/users/1/recommendations?algo=magic", None).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(call(&app, "GET", "/api/users/99999/recommendations", None).await.0, StatusCode::NOT_FOUND);
    let unknown = json!({ "name": "x", "genres": ["Cooking Battle"] });
    assert_eq!(call(&app, "POST", "/api/users", Some(unknown)).await.0, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn eval_endpoint_reports_every_algorithm() {
    let app = app().await;
    let (_, rows) = call(&app, "GET", "/api/eval", None).await;
    let names: Vec<&str> = rows.as_array().expect("rows").iter().filter_map(|r| r["algorithm"].as_str()).collect();
    for algo in ["popular", "cf", "content", "hybrid"] {
        assert!(names.contains(&algo));
    }
}
