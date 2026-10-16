use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use drm::audit::AuditLog;
use drm::crypto::{b64url_encode, hex_decode};
use drm::http::{AppState, now, router};
use drm::keystore::MemoryKeyStore;
use drm::license::{ClearKeyService, LicenseService};
use drm::session::SessionSigner;
use drm::url_signer::UrlSigner;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

const KID: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f90";
const KEY: &str = "0123456789abcdef0123456789abcdef";

fn media_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vp12-{name}-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("clip")).unwrap();
    std::fs::write(dir.join("clip/stream.mpd"), "<MPD/>").unwrap();
    std::fs::write(dir.join("clip/chunk-stream0-00001.m4s"), [1u8, 2, 3]).unwrap();
    dir
}

fn app(name: &str) -> Router {
    let store = Arc::new(MemoryKeyStore::from_hex(&[("clip".into(), KID.into(), KEY.into())]).unwrap());
    let mut licenses: HashMap<String, Arc<dyn LicenseService>> = HashMap::new();
    licenses.insert("clearkey".into(), Arc::new(ClearKeyService::new(store)));
    router(AppState {
        sessions: SessionSigner::new(b"session".to_vec()),
        urls: UrlSigner::new(b"urls".to_vec()),
        licenses,
        media_dir: media_dir(name),
        audit: AuditLog::new(10),
    })
}

async fn call(app: &Router, method: &str, uri: &str, token: Option<&str>, body: Value) -> (StatusCode, Value, Vec<u8>) {
    let mut req = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    let body = if method == "GET" { Body::empty() } else { Body::from(body.to_string()) };
    let res = app.clone().oneshot(req.body(body).unwrap()).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes().to_vec();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json, bytes)
}

async fn session(app: &Router, entitled: bool, expired: bool) -> String {
    let (_, body, _) = call(app, "POST", "/api/session", None, json!({"user":"alice","entitled":entitled,"expired":expired})).await;
    body["token"].as_str().unwrap().to_string()
}

async fn manifest(app: &Router, token: &str, expired: bool) -> String {
    let (status, body, _) = call(app, "POST", "/api/playback", Some(token), json!({"content":"clip","expired":expired})).await;
    assert_eq!(status, StatusCode::OK);
    body["manifestUrl"].as_str().unwrap().to_string()
}

fn kid_request() -> Value {
    json!({"kids":[b64url_encode(&hex_decode(KID).unwrap())],"type":"temporary"})
}

#[tokio::test]
async fn entitled_viewer_gets_manifest_segments_and_the_content_key() {
    let app = app("happy");
    let token = session(&app, true, false).await;
    let url = manifest(&app, &token, false).await;
    let (status, _, bytes) = call(&app, "GET", &url, None, Value::Null).await;
    assert_eq!((status, bytes), (StatusCode::OK, b"<MPD/>".to_vec()));
    let segment = url.replace("stream.mpd", "chunk-stream0-00001.m4s");
    assert_eq!(call(&app, "GET", &segment, None, Value::Null).await.0, StatusCode::OK);
    let (status, license, _) = call(&app, "POST", "/api/license/clearkey", Some(&token), kid_request()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(license["keys"][0]["k"], b64url_encode(&hex_decode(KEY).unwrap()));
}

#[tokio::test]
async fn expired_url_token_blocks_segment_delivery() {
    let app = app("expired-url");
    let token = session(&app, true, false).await;
    let url = manifest(&app, &token, true).await;
    let (status, body, _) = call(&app, "GET", &url, None, Value::Null).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::FORBIDDEN, Some("expired")));
}

#[tokio::test]
async fn tampered_url_signature_and_extended_expiry_are_rejected() {
    let app = app("tamper");
    let token = session(&app, true, false).await;
    let url = manifest(&app, &token, false).await;
    let parts: Vec<&str> = url.split('/').collect();
    let extended = url.replacen(parts[2], &(now() + 99_999).to_string(), 1);
    let (status, body, _) = call(&app, "GET", &extended, None, Value::Null).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::FORBIDDEN, Some("bad_signature")));
    let sig = parts[3];
    let flipped = format!("{}{}", if sig.starts_with('0') { "1" } else { "0" }, &sig[1..]);
    let (status, _, _) = call(&app, "GET", &url.replacen(sig, &flipped, 1), None, Value::Null).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn viewer_without_entitlement_streams_ciphertext_but_gets_no_key() {
    let app = app("no-ent");
    let token = session(&app, false, false).await;
    let (status, body, _) = call(&app, "POST", "/api/license/clearkey", Some(&token), kid_request()).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::FORBIDDEN, Some("not_entitled")));
}

#[tokio::test]
async fn license_requires_a_live_session() {
    let app = app("no-session");
    let (status, _, _) = call(&app, "POST", "/api/license/clearkey", None, kid_request()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let expired = session(&app, true, true).await;
    let (status, body, _) = call(&app, "POST", "/api/license/clearkey", Some(&expired), kid_request()).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::FORBIDDEN, Some("expired")));
}

#[tokio::test]
async fn unregistered_key_system_is_not_found() {
    let app = app("system");
    let token = session(&app, true, false).await;
    let (status, _, _) = call(&app, "POST", "/api/license/widevine", Some(&token), kid_request()).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn path_traversal_in_media_file_is_refused() {
    let app = app("traversal");
    let token = session(&app, true, false).await;
    let url = manifest(&app, &token, false).await;
    let (status, _, _) = call(&app, "GET", &url.replace("stream.mpd", "..%2Fdrm.json"), None, Value::Null).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn audit_log_records_denials_for_operators() {
    let app = app("audit");
    let token = session(&app, false, false).await;
    call(&app, "POST", "/api/license/clearkey", Some(&token), kid_request()).await;
    let (_, events, _) = call(&app, "GET", "/api/audit", None, Value::Null).await;
    assert_eq!(events[0]["kind"], "license:clearkey");
    assert_eq!(events[0]["outcome"], "denied");
}
