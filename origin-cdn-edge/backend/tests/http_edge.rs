use cdn::cache::LruCache;
use cdn::edge::Edge;
use cdn::origin_client::HttpOrigin;
use cdn::origin_server::{OriginConfig, OriginState};
use std::collections::HashMap;
use std::sync::Arc;

async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

async fn stack(latency_ms: u64, max_age: u64, swr: u64, capacity: usize) -> (String, Arc<OriginState>) {
    let origin = OriginState::new(OriginConfig {
        latency_ms,
        segment_bytes: 4096,
        max_age,
        stale_while_revalidate: swr,
    });
    let origin_url = serve(cdn::origin_server::router(origin.clone())).await;
    let edge = Edge::new(LruCache::new(capacity), HttpOrigin::new(&origin_url));
    let edge_url = serve(cdn::edge_server::router(edge)).await;
    (edge_url, origin)
}

async fn fetch(client: &reqwest::Client, url: String) -> (String, Vec<u8>) {
    let response = client.get(url).send().await.unwrap();
    assert!(response.status().is_success());
    let cache = response.headers()["x-cache"].to_str().unwrap().to_string();
    (cache, response.bytes().await.unwrap().to_vec())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fifty_concurrent_http_misses_reach_origin_once() {
    let (edge_url, origin) = stack(300, 60, 60, 1 << 20).await;
    let client = reqwest::Client::new();
    let tasks: Vec<_> = (0..50)
        .map(|_| {
            let client = client.clone();
            let url = format!("{edge_url}/seg/hot.ts");
            tokio::spawn(async move { fetch(&client, url).await })
        })
        .collect();
    let mut statuses: HashMap<String, usize> = HashMap::new();
    let mut bodies = Vec::new();
    for task in tasks {
        let (cache, body) = task.await.unwrap();
        *statuses.entry(cache).or_default() += 1;
        bodies.push(body);
    }
    assert_eq!(origin.stats().requests, 1, "statuses {statuses:?}");
    assert!(bodies.windows(2).all(|w| w[0] == w[1]));
    assert_eq!(bodies[0].len(), 4096);

    let (cache, _) = fetch(&client, format!("{edge_url}/seg/hot.ts")).await;
    assert_eq!(cache, "HIT");

    let metrics: serde_json::Value =
        serde_json::from_str(&client.get(format!("{edge_url}/metrics")).send().await.unwrap().text().await.unwrap())
            .unwrap();
    assert_eq!(metrics["origin_fetches"], 1);
    assert_eq!(metrics["requests"], 51);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stale_header_then_refreshed_body_over_http() {
    let (edge_url, origin) = stack(50, 1, 30, 1 << 20).await;
    let client = reqwest::Client::new();
    let url = format!("{edge_url}/seg/live.ts");
    let (first, v1) = fetch(&client, url.clone()).await;
    assert_eq!(first, "MISS");
    assert!(v1.starts_with(b"live.ts v1"));

    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let (second, stale_body) = fetch(&client, url.clone()).await;
    assert_eq!(second, "STALE");
    assert_eq!(stale_body, v1);

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert_eq!(origin.stats().requests, 2);
    let (third, fresh_body) = fetch(&client, url).await;
    assert_eq!(third, "HIT");
    assert!(fresh_body.starts_with(b"live.ts v2"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn probe_endpoint_reports_one_leader_and_origin_confirms_one_fetch_for_the_key() {
    let (edge_url, _) = stack(200, 60, 60, 1 << 20).await;
    let client = reqwest::Client::new();
    let report: serde_json::Value = serde_json::from_str(
        &client.post(format!("{edge_url}/probe/cold.ts?n=100")).send().await.unwrap().text().await.unwrap(),
    )
    .unwrap();
    let results = report["results"].as_array().unwrap();
    assert_eq!(results.len(), 100);
    assert!(results.iter().all(|r| r["status"] == "MISS"));
    assert_eq!(results.iter().filter(|r| r["coalesced"] == true).count(), 99);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn origin_counts_fetches_per_key() {
    let (edge_url, origin) = stack(10, 60, 60, 1 << 20).await;
    let client = reqwest::Client::new();
    fetch(&client, format!("{edge_url}/seg/a.ts")).await;
    fetch(&client, format!("{edge_url}/seg/a.ts")).await;
    fetch(&client, format!("{edge_url}/seg/b.ts")).await;
    assert_eq!(origin.stats().requests, 2);
    assert_eq!(origin.stats().unique_keys, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unreachable_origin_becomes_bad_gateway() {
    let edge = Edge::new(LruCache::new(1024), HttpOrigin::new("http://127.0.0.1:1"));
    let edge_url = serve(cdn::edge_server::router(edge)).await;
    let response = reqwest::get(format!("{edge_url}/seg/x.ts")).await.unwrap();
    assert_eq!(response.status().as_u16(), 502);
}
