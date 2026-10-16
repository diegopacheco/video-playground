use crate::blocking::{self, BlockRequest};
use crate::error::{Error, Result};
use crate::live::LiveStream;
use crate::naming::{self, Mode, Resource};
use crate::playlist::PlaylistBuilder;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub struct AppState {
    pub live: Arc<LiveStream>,
    pub low_latency: Box<dyn PlaylistBuilder>,
    pub regular: Box<dyn PlaylistBuilder>,
    pub block_timeout: Duration,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new().route("/live/{file}", get(serve)).with_state(state)
}

async fn serve(
    State(state): State<Arc<AppState>>,
    Path(file): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    match handle(&state, &file, &query).await {
        Ok(response) => response,
        Err(error) => error_response(error),
    }
}

async fn handle(state: &AppState, file: &str, query: &HashMap<String, String>) -> Result<Response> {
    match naming::parse(file)? {
        Resource::Playlist(Mode::LowLatency) => {
            if let Some(req) = blocking::parse(query)? {
                state.live.wait_for(req, state.block_timeout).await?;
            }
            Ok(playlist(state.live.read(|w| state.low_latency.build(w))))
        }
        Resource::Playlist(Mode::Regular) => Ok(playlist(state.live.read(|w| state.regular.build(w)))),
        Resource::Init => state.live.read(|w| w.init.clone()).map(media).ok_or(Error::NotFound),
        Resource::Segment(msn) => state
            .live
            .read(|w| w.segment(msn).filter(|s| s.complete).map(|s| s.data()))
            .map(media)
            .ok_or(Error::NotFound),
        Resource::Part(msn, index) => {
            let req = BlockRequest { msn, part: Some(index) };
            match state.live.wait_for(req, state.block_timeout).await {
                Err(Error::BadRequest(_)) => return Err(Error::NotFound),
                other => other?,
            }
            state
                .live
                .read(|w| w.part(msn, index).map(|p| p.data.clone()))
                .map(media)
                .ok_or(Error::NotFound)
        }
    }
}

fn playlist(body: String) -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/vnd.apple.mpegurl"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
        .into_response()
}

fn media(body: Bytes) -> Response {
    (
        [(header::CONTENT_TYPE, "video/mp4"), (header::CACHE_CONTROL, "max-age=60")],
        body,
    )
        .into_response()
}

fn error_response(error: Error) -> Response {
    let status = match error {
        Error::BadRequest(_) => StatusCode::BAD_REQUEST,
        Error::NotFound => StatusCode::NOT_FOUND,
        Error::Timeout => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string()).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::fixture::{config, feed, part};
    use crate::playlist::{LowLatencyPlaylist, RegularPlaylist};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    async fn spawn(live: Arc<LiveStream>) -> String {
        let state = Arc::new(AppState {
            live,
            low_latency: Box::new(LowLatencyPlaylist { part_hold_back_factor: 3.0, part_window_targets: 3.0 }),
            regular: Box::new(RegularPlaylist { hold_back_segments: 3.0 }),
            block_timeout: Duration::from_millis(300),
        });
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        tokio::spawn(async move { axum::serve(listener, router(state)).await.unwrap() });
        addr
    }

    async fn get(addr: &str, path: &str) -> (u16, String) {
        let mut stream = TcpStream::connect(addr).await.unwrap();
        let request = format!("GET {path} HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut raw = String::new();
        stream.read_to_string(&mut raw).await.unwrap();
        let status = raw[9..12].parse().unwrap();
        (status, raw)
    }

    #[tokio::test]
    async fn blocked_playlist_request_is_answered_when_the_part_arrives() {
        let live = Arc::new(LiveStream::new(config()));
        live.update(|w| {
            w.init = Some(Bytes::from_static(b"init"));
            feed(w, 12);
        });
        let addr = spawn(live.clone()).await;
        let pending = tokio::spawn({
            let addr = addr.clone();
            async move { get(&addr, "/live/ll.m3u8?_HLS_msn=1&_HLS_part=2").await }
        });
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!pending.is_finished());
        live.update(|w| w.push_part(part(false), 0));
        let (status, body) = pending.await.unwrap();
        assert_eq!(status, 200);
        assert!(body.contains("part1.2.m4s\""));
        assert!(body.contains("PRELOAD-HINT:TYPE=PART,URI=\"part1.3.m4s\""));
    }

    #[tokio::test]
    async fn blocking_errors_map_to_spec_status_codes() {
        let live = Arc::new(LiveStream::new(config()));
        live.update(|w| feed(w, 12));
        let addr = spawn(live).await;
        assert_eq!(get(&addr, "/live/ll.m3u8?_HLS_part=1").await.0, 400);
        assert_eq!(get(&addr, "/live/ll.m3u8?_HLS_msn=9").await.0, 400);
        assert_eq!(get(&addr, "/live/ll.m3u8?_HLS_msn=2").await.0, 503);
    }

    #[tokio::test]
    async fn media_is_served_only_once_it_is_complete() {
        let live = Arc::new(LiveStream::new(config()));
        live.update(|w| feed(w, 12));
        let addr = spawn(live).await;
        assert_eq!(get(&addr, "/live/seg0.m4s").await.0, 200);
        assert_eq!(get(&addr, "/live/seg1.m4s").await.0, 404);
        assert_eq!(get(&addr, "/live/part1.1.m4s").await.0, 200);
        assert_eq!(get(&addr, "/live/init.mp4").await.0, 404);
        assert_eq!(get(&addr, "/live/nope.txt").await.0, 404);
        let (status, body) = get(&addr, "/live/hls.m3u8").await;
        assert_eq!(status, 200);
        assert!(body.contains("application/vnd.apple.mpegurl"));
    }
}
