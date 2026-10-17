use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::Stream;
use futures::stream::unfold;
use serde::Serialize;
use tokio::sync::broadcast::error::RecvError;

use crate::aggregator::WindowResult;
use crate::beacon::Beacon;
use crate::error::AppError;
use crate::hub::Hub;
use crate::sink::BeaconSink;

pub struct AppState<S> {
    pub sink: Arc<S>,
    pub hub: Arc<Hub>,
}

impl<S> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            sink: self.sink.clone(),
            hub: self.hub.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct Accepted {
    pub accepted: usize,
}

pub fn router<S: BeaconSink>(state: AppState<S>) -> Router {
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/beacons", post(ingest::<S>))
        .route("/api/windows", get(windows::<S>))
        .route("/api/stream", get(stream::<S>))
        .with_state(state)
}

async fn ingest<S: BeaconSink>(
    State(state): State<AppState<S>>,
    Json(beacons): Json<Vec<Beacon>>,
) -> Result<(StatusCode, Json<Accepted>), AppError> {
    for beacon in &beacons {
        beacon.validate()?;
    }
    let accepted = state.sink.publish(&beacons).await?;
    Ok((StatusCode::ACCEPTED, Json(Accepted { accepted })))
}

async fn windows<S: BeaconSink>(State(state): State<AppState<S>>) -> Json<Vec<WindowResult>> {
    Json(state.hub.history())
}

async fn stream<S: BeaconSink>(
    State(state): State<AppState<S>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.hub.subscribe();
    let events = unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(batch) => {
                    let event = Event::default().event("windows").json_data(&batch).ok()?;
                    return Some((Ok(event), rx));
                }
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return None,
            }
        }
    });
    Sse::new(events).keep_alive(KeepAlive::default())
}
