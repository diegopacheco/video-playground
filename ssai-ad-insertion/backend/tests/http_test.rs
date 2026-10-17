mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn write_creative(dir: &Path, id: &str, segments: usize) {
    let folder = dir.join("ads").join(id);
    std::fs::create_dir_all(&folder).unwrap();
    let mut playlist = String::from("#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:2\n");
    for i in 0..segments {
        playlist.push_str(&format!("#EXTINF:2.000000,\nseg_{i:03}.ts\n"));
        std::fs::write(folder.join(format!("seg_{i:03}.ts")), format!("{id}-{i}")).unwrap();
    }
    playlist.push_str("#EXT-X-ENDLIST\n");
    std::fs::write(folder.join("index.m3u8"), playlist).unwrap();
}

fn media_dir() -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("ssai-http-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(dir.join("content")).unwrap();
    std::fs::write(dir.join("content").join("content.m3u8"), common::CONTENT).unwrap();
    std::fs::write(dir.join("content").join("seg_000.ts"), "content-0").unwrap();
    for (id, segments) in [("acme", 3), ("globex", 2), ("initech", 4), ("umbrella", 2), ("slate", 1)] {
        write_creative(&dir, id, segments);
    }
    dir
}

fn app() -> Router {
    ssai::http::router(Arc::new(ssai::build_state(&media_dir()).unwrap()))
}

async fn get(app: &Router, uri: &str) -> (StatusCode, String) {
    let response = app.clone().oneshot(Request::get(uri).body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&body).to_string())
}

#[tokio::test]
async fn fetching_every_ad_segment_records_the_full_beacon_sequence() {
    let app = app();
    let (status, manifest) = get(&app, "/api/manifest.m3u8?user=alice&session=t1").await;
    assert_eq!(status, StatusCode::OK);
    let tracked: Vec<&str> = manifest.lines().filter(|l| l.starts_with("/api/track/")).collect();
    assert!(!tracked.is_empty());
    for uri in &tracked {
        let (status, body) = get(&app, uri).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains('-'));
    }
    for uri in &tracked {
        get(&app, uri).await;
    }
    let (_, beacons) = get(&app, "/api/beacons?session=t1").await;
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&beacons).unwrap();
    let pods: std::collections::HashSet<(String, String)> = tracked
        .iter()
        .map(|u| {
            let p: Vec<&str> = u.split('/').collect();
            (p[5].to_string(), p[6].to_string())
        })
        .collect();
    assert_eq!(parsed.len(), pods.len() * 5, "five events per ad play, refetches are not counted");
    assert!(parsed.iter().all(|b| b["user"] == "alice"));
}

#[tokio::test]
async fn premium_manifest_has_no_ads_and_no_tracking() {
    let app = app();
    let (status, manifest) = get(&app, "/api/manifest.m3u8?user=diana").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!manifest.contains("/api/track/"));
    assert!(!manifest.contains("#EXT-X-DISCONTINUITY"));
    assert!(manifest.contains("/media/content/seg_000.ts"));
}

#[tokio::test]
async fn bad_requests_are_rejected() {
    let app = app();
    assert_eq!(get(&app, "/api/manifest.m3u8?user=nobody").await.0, StatusCode::NOT_FOUND);
    assert_eq!(get(&app, "/api/manifest.m3u8?user=alice&session=a/b").await.0, StatusCode::NOT_FOUND);
    assert_eq!(get(&app, "/media/../Cargo.toml").await.0, StatusCode::NOT_FOUND);
    assert_eq!(get(&app, "/api/track/alice/s/b/acme/99.ts").await.0, StatusCode::NOT_FOUND);
    assert_eq!(get(&app, "/media/content/seg_000.ts").await.1, "content-0");
}

#[tokio::test]
async fn stitch_view_exposes_raw_and_stitched_side_by_side() {
    let app = app();
    let (status, body) = get(&app, "/api/stitch?user=bruno&session=v1").await;
    assert_eq!(status, StatusCode::OK);
    let view: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(view["raw"].as_str().unwrap().contains("#EXT-X-CUE-OUT"));
    assert!(view["stitched"].as_str().unwrap().contains("#EXT-X-DISCONTINUITY"));
    assert_eq!(view["breaks"].as_array().unwrap().len(), 2);
}
