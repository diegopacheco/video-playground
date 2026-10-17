use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use resume_watching::model::Playhead;
use resume_watching::row::continue_watching;
use resume_watching::store::{self, PlayheadStore};

fn redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6110".into())
}

fn scylla_node() -> String {
    std::env::var("SCYLLA_NODE").unwrap_or_else(|_| "127.0.0.1:6210".into())
}

static NEXT: AtomicU64 = AtomicU64::new(0);

fn unique(prefix: &str) -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{prefix}-{nanos}-{}", NEXT.fetch_add(1, Ordering::SeqCst))
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

fn hb(profile: &str, title: &str, device: &str, position: u32, ts: i64) -> Playhead {
    Playhead {
        user_id: "u-test".into(),
        profile_id: profile.into(),
        device_id: device.into(),
        title_id: title.into(),
        episode_id: "s1e1".into(),
        position_secs: position,
        duration_secs: 1000,
        client_ts_ms: ts,
    }
}

async fn open(kind: &str) -> Box<dyn PlayheadStore> {
    store::connect(kind, &redis_url(), &scylla_node())
        .await
        .unwrap_or_else(|e| panic!("{kind} must be running for store tests, start it with ./scripts/start-all.sh: {e}"))
}

async fn late_arriving_older_write_does_not_overwrite_newer(kind: &str) {
    let s = open(kind).await;
    let profile = unique(kind);
    let now = now_ms();
    s.upsert(&hb(&profile, "t1", "phone", 600, now + 5_000)).await.unwrap();
    s.upsert(&hb(&profile, "t1", "tv", 100, now)).await.unwrap();
    let rows = s.list(&profile).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].device_id, "phone", "{kind}: the TV heartbeat arrived later but was taken earlier, it must lose");
    assert_eq!(rows[0].position_secs, 600);
    s.clear(&profile).await.unwrap();
}

async fn newer_write_from_other_device_replaces_row(kind: &str) {
    let s = open(kind).await;
    let profile = unique(kind);
    let now = now_ms();
    s.upsert(&hb(&profile, "t1", "tv", 800, now)).await.unwrap();
    s.upsert(&hb(&profile, "t1", "phone", 50, now + 1)).await.unwrap();
    let rows = s.list(&profile).await.unwrap();
    assert_eq!(rows[0].device_id, "phone");
    assert_eq!(rows[0].position_secs, 50);
    s.clear(&profile).await.unwrap();
}

async fn row_from_store_is_sorted_and_hides_finished(kind: &str) {
    let s = open(kind).await;
    let profile = unique(kind);
    let now = now_ms();
    s.upsert(&hb(&profile, "a", "tv", 100, now)).await.unwrap();
    s.upsert(&hb(&profile, "b", "tv", 200, now + 10)).await.unwrap();
    s.upsert(&hb(&profile, "c", "tv", 999, now + 20)).await.unwrap();
    let row = continue_watching(s.list(&profile).await.unwrap(), vec![]);
    let titles: Vec<&str> = row.iter().map(|r| r.playhead.title_id.as_str()).collect();
    assert_eq!(titles, vec!["b", "a"]);
    s.clear(&profile).await.unwrap();
    assert!(s.list(&profile).await.unwrap().is_empty());
}

#[tokio::test]
async fn redis_lww_rejects_older_client_timestamp() {
    late_arriving_older_write_does_not_overwrite_newer("redis").await;
}

#[tokio::test]
async fn scylla_lww_rejects_older_client_timestamp() {
    late_arriving_older_write_does_not_overwrite_newer("scylla").await;
}

#[tokio::test]
async fn redis_newer_device_wins() {
    newer_write_from_other_device_replaces_row("redis").await;
}

#[tokio::test]
async fn scylla_newer_device_wins() {
    newer_write_from_other_device_replaces_row("scylla").await;
}

#[tokio::test]
async fn redis_continue_watching_row() {
    row_from_store_is_sorted_and_hides_finished("redis").await;
}

#[tokio::test]
async fn scylla_continue_watching_row() {
    row_from_store_is_sorted_and_hides_finished("scylla").await;
}
