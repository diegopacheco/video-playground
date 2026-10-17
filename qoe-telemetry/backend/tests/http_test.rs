use std::sync::Arc;

use qoe::aggregator::{Dimension, WindowMetrics, WindowResult};
use qoe::beacon::{Beacon, BeaconKind};
use qoe::http::{AppState, router};
use qoe::hub::{Hub, WindowBatch};
use qoe::sink::MemorySink;
use serde_json::json;

async fn serve() -> (String, Arc<MemorySink>, Arc<Hub>) {
    let sink = Arc::new(MemorySink::default());
    let hub = Arc::new(Hub::new(10));
    let app = router(AppState {
        sink: sink.clone(),
        hub: hub.clone(),
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move { axum::serve(listener, app).await });
    (format!("http://{addr}"), sink, hub)
}

#[tokio::test]
async fn player_beacon_contract_is_accepted_and_forwarded_to_the_sink() {
    let (base, sink, _) = serve().await;
    let body = json!([
        {"session_id": "p1", "title": "local-clip", "cdn": "local", "ts": 1_700_000_000_000i64, "kind": "startup", "startup_ms": 850},
        {"session_id": "p1", "title": "local-clip", "cdn": "local", "ts": 1_700_000_000_500i64, "kind": "bitrate_switch", "from_kbps": 800, "to_kbps": 1800},
        {"session_id": "p1", "title": "local-clip", "cdn": "local", "ts": 1_700_000_001_000i64, "kind": "rebuffer_start"}
    ]);
    let resp = reqwest::Client::new()
        .post(format!("{base}/api/beacons"))
        .json(&body)
        .send()
        .await
        .expect("post");
    assert_eq!(resp.status(), 202);
    let stored = sink.stored();
    assert_eq!(stored.len(), 3);
    assert_eq!(stored[0].event, BeaconKind::Startup { startup_ms: 850 });
    assert_eq!(stored[2].event, BeaconKind::RebufferStart);
}

#[tokio::test]
async fn a_batch_with_one_invalid_beacon_is_rejected_whole_so_kafka_never_sees_garbage() {
    let (base, sink, _) = serve().await;
    let body = json!([
        {"session_id": "p1", "title": "t", "cdn": "c", "ts": 1_700_000_000_000i64, "kind": "heartbeat", "played_ms": 1000},
        {"session_id": "", "title": "t", "cdn": "c", "ts": 1_700_000_000_000i64, "kind": "heartbeat", "played_ms": 1000}
    ]);
    let resp = reqwest::Client::new()
        .post(format!("{base}/api/beacons"))
        .json(&body)
        .send()
        .await
        .expect("post");
    assert_eq!(resp.status(), 400);
    assert!(sink.stored().is_empty());
}

#[tokio::test]
async fn unknown_beacon_kind_is_a_client_error() {
    let (base, _, _) = serve().await;
    let body = json!([{"session_id": "p", "title": "t", "cdn": "c", "ts": 1i64, "kind": "teleport"}]);
    let resp = reqwest::Client::new()
        .post(format!("{base}/api/beacons"))
        .json(&body)
        .send()
        .await
        .expect("post");
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn closed_windows_are_kept_in_history_for_dashboards_that_connect_late() {
    let (base, _, hub) = serve().await;
    let result = WindowResult {
        window_start: 0,
        window_end: 10_000,
        dimension: Dimension::Cdn,
        key: "cdn-gamma".into(),
        metrics: WindowMetrics {
            sessions: 1,
            events: 1,
            startups: 1,
            avg_startup_ms: 5_000.0,
            p95_startup_ms: 5_000.0,
            rebuffer_ratio: 0.2,
            switches_per_min: 4.0,
            error_rate: 0.0,
            played_ms: 8_000,
            rebuffer_ms: 2_000,
        },
    };
    hub.publish(WindowBatch {
        results: vec![result],
        consumed: 1,
        late_dropped: 0,
    });
    let history: serde_json::Value = reqwest::get(format!("{base}/api/windows"))
        .await
        .expect("get")
        .json()
        .await
        .expect("json");
    assert_eq!(history[0]["key"], "cdn-gamma");
    assert_eq!(history[0]["dimension"], "cdn");
}

#[test]
fn beacon_json_roundtrip_keeps_the_kind_tag_flat() {
    let beacon = Beacon {
        session_id: "s".into(),
        title: "t".into(),
        cdn: "c".into(),
        ts: 5,
        event: BeaconKind::Error { code: "E1".into() },
    };
    let raw = beacon.to_json().expect("encode");
    assert!(raw.contains("\"kind\":\"error\""));
    assert_eq!(Beacon::from_json(raw.as_bytes()).expect("decode"), beacon);
}
