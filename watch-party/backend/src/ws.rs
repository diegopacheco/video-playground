use crate::error::RoomError;
use crate::hub::{Hub, Outbox};
use crate::playback::PlaybackState;
use crate::protocol::{ClientMsg, ServerMsg};
use crate::room::MemberId;
use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use std::sync::Arc;
use tokio::sync::mpsc;

pub fn router(hub: Arc<Hub>) -> Router {
    Router::new()
        .route("/ws", get(upgrade))
        .route("/health", get(|| async { "ok" }))
        .with_state(hub)
}

async fn upgrade(ws: WebSocketUpgrade, State(hub): State<Arc<Hub>>) -> Response {
    ws.on_upgrade(move |socket| session(socket, hub))
}

struct Seat {
    room: String,
    id: MemberId,
}

async fn send(socket: &mut WebSocket, msg: &ServerMsg) -> bool {
    match serde_json::to_string(msg) {
        Ok(text) => socket.send(Message::Text(text.into())).await.is_ok(),
        Err(_) => true,
    }
}

async fn session(mut socket: WebSocket, hub: Arc<Hub>) {
    let (outbox, mut inbox) = mpsc::unbounded_channel();
    let mut seat: Option<Seat> = None;
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let Some(Ok(message)) = incoming else { break };
                let t1 = hub.now_ms();
                let text = match message {
                    Message::Text(text) => text,
                    Message::Close(_) => break,
                    _ => continue,
                };
                match serde_json::from_str::<ClientMsg>(&text) {
                    Ok(ClientMsg::Ping { t0 }) => {
                        let pong = ServerMsg::Pong { t0, t1, t2: hub.now_ms() };
                        if !send(&mut socket, &pong).await { break; }
                    }
                    Ok(msg) => {
                        if let Err(e) = handle(&hub, &outbox, &mut seat, msg) {
                            let _ = outbox.send(ServerMsg::Error { message: e.to_string() });
                        }
                    }
                    Err(e) => {
                        let _ = outbox.send(ServerMsg::Error { message: format!("bad message: {e}") });
                    }
                }
            }
            outgoing = inbox.recv() => {
                let Some(msg) = outgoing else { break };
                if !send(&mut socket, &msg).await { break; }
            }
        }
    }
    if let Some(seat) = seat {
        let _ = hub.leave(&seat.room, seat.id);
    }
}

fn handle(hub: &Hub, outbox: &Outbox, seat: &mut Option<Seat>, msg: ClientMsg) -> Result<(), RoomError> {
    match msg {
        ClientMsg::Join { room, name } => {
            if seat.is_some() {
                return Err(RoomError::InvalidJoin("already joined a room"));
            }
            let room = room.trim().to_string();
            let id = hub.join(&room, &name, outbox.clone())?;
            *seat = Some(Seat { room, id });
            Ok(())
        }
        ClientMsg::State { playing, position, rate, at } => {
            let seat = seat.as_ref().ok_or(RoomError::InvalidJoin("join a room first"))?;
            hub.set_state(&seat.room, seat.id, PlaybackState { playing, position, rate, at })
        }
        ClientMsg::Ping { .. } => Ok(()),
    }
}
