use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use watch_party::clock::SystemClock;
use watch_party::hub::Hub;
use watch_party::protocol::{ClientMsg, ServerMsg};
use watch_party::room::InMemoryRooms;
use watch_party::ws::router;

type Client = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn start() -> String {
    let hub = Arc::new(Hub::new(Box::new(InMemoryRooms::new()), Arc::new(SystemClock)));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router(hub)).await.unwrap() });
    format!("ws://{addr}/ws")
}

async fn send(client: &mut Client, msg: ClientMsg) {
    let text = serde_json::to_string(&msg).unwrap();
    client.send(Message::Text(text.into())).await.unwrap();
}

async fn next(client: &mut Client) -> ServerMsg {
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(5), client.next())
            .await
            .expect("server answered in time")
            .expect("socket open")
            .unwrap();
        if let Message::Text(text) = frame {
            return serde_json::from_str(&text).unwrap();
        }
    }
}

async fn join(url: &str, name: &str) -> (Client, u64) {
    let (mut client, _) = connect_async(url).await.unwrap();
    send(&mut client, ClientMsg::Join { room: "movie".into(), name: name.into() }).await;
    let ServerMsg::Welcome { you } = next(&mut client).await else { panic!("welcome first") };
    (client, you)
}

async fn next_room(client: &mut Client) -> watch_party::room::RoomView {
    loop {
        if let ServerMsg::Room { view } = next(client).await {
            return view;
        }
    }
}

#[tokio::test]
async fn pong_carries_server_receive_and_send_times_for_ntp_offset() {
    let url = start().await;
    let (mut client, _) = join(&url, "alice").await;
    let _ = next_room(&mut client).await;
    send(&mut client, ClientMsg::Ping { t0: 123.0 }).await;
    let ServerMsg::Pong { t0, t1, t2 } = next(&mut client).await else { panic!("pong expected") };
    assert_eq!(t0, 123.0);
    assert!(t1 > 1.0e12);
    assert!(t2 >= t1);
}

#[tokio::test]
async fn leader_state_reaches_followers_and_disconnect_hands_off_leadership() {
    let url = start().await;
    let (mut alice, alice_id) = join(&url, "alice").await;
    assert_eq!(next_room(&mut alice).await.leader, alice_id);
    let (mut bob, bob_id) = join(&url, "bob").await;
    let view = next_room(&mut bob).await;
    assert_eq!(view.leader, alice_id);
    assert_eq!(view.members.len(), 2);
    let _ = next_room(&mut alice).await;

    send(&mut bob, ClientMsg::State { playing: true, position: 99.0, rate: 1.0, at: 0.0 }).await;
    let ServerMsg::Error { message } = next(&mut bob).await else { panic!("follower must be refused") };
    assert!(message.contains("leader"));

    send(&mut alice, ClientMsg::State { playing: true, position: 12.0, rate: 1.0, at: 1.0e12 }).await;
    let view = next_room(&mut bob).await;
    assert!(view.state.playing);
    assert_eq!(view.state.position, 12.0);

    alice.close(None).await.unwrap();
    let view = next_room(&mut bob).await;
    assert_eq!(view.leader, bob_id);
    assert_eq!(view.members.len(), 1);
    assert!(view.state.playing);
    assert!(view.state.position > 12.0);
}
