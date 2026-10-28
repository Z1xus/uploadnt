use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::get;
use lobby::{MAX_MESSAGE, encode, valid_mailbox};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tower_http::services::{ServeDir, ServeFile};
use uploadnt_core::Lobby;

type Rooms = Arc<Mutex<HashMap<String, Vec<(u64, UnboundedSender<Vec<u8>>)>>>>;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
const PING: Duration = Duration::from_secs(20);
const DEAD: Duration = Duration::from_secs(60);

#[tokio::main]
pub async fn main() {
    let site = std::env::var("SITE").unwrap_or_else(|_| "web/build".into());
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "0.0.0.0:8787".into());
    let pages = ServeDir::new(&site).fallback(ServeFile::new(format!("{site}/index.html")));
    let app = Router::new().route("/m/{mailbox}", get(join)).fallback_service(pages).with_state(Rooms::default());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("lobby on {addr}, serving {site}");
    axum::serve(listener, app).await.unwrap();
}

async fn join(Path(mailbox): Path<String>, State(rooms): State<Rooms>, ws: WebSocketUpgrade) -> Response {
    ws.max_message_size(MAX_MESSAGE).on_upgrade(move |socket| run(socket, mailbox, rooms))
}

async fn run(mut socket: WebSocket, mailbox: String, rooms: Rooms) {
    if !valid_mailbox(&mailbox) {
        return;
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, mut rx) = unbounded_channel();
    let joined = {
        let mut rooms = rooms.lock().unwrap();
        let room = rooms.entry(mailbox.clone()).or_default();
        if room.len() < 2 {
            room.push((id, tx));
            if room.len() == 2 {
                room.iter().for_each(|(_, peer)| _ = peer.send(encode(&Lobby::Peer(true))));
            }
            true
        } else {
            false
        }
    };
    if !joined {
        _ = socket.send(Message::Binary(encode(&Lobby::Full).into())).await;
        return;
    }

    let mut ping = tokio::time::interval(PING);
    let mut seen = Instant::now();
    loop {
        tokio::select! {
            msg = socket.recv() => {
                seen = Instant::now();
                match msg {
                    Some(Ok(Message::Binary(bytes))) => {
                        let relay = encode(&Lobby::Relay(bytes.to_vec()));
                        let rooms = rooms.lock().unwrap();
                        rooms[&mailbox].iter().filter(|(peer, _)| *peer != id).for_each(|(_, peer)| _ = peer.send(relay.clone()));
                    }
                    Some(Ok(_)) => {}
                    _ => break,
                }
            }
            Some(out) = rx.recv() => {
                if socket.send(Message::Binary(out.into())).await.is_err() {
                    break;
                }
            }
            _ = ping.tick() => {
                if seen.elapsed() > DEAD || socket.send(Message::Ping(Default::default())).await.is_err() {
                    break;
                }
            }
        }
    }

    let mut rooms = rooms.lock().unwrap();
    let room = rooms.get_mut(&mailbox).unwrap();
    room.retain(|(peer, _)| *peer != id);
    room.iter().for_each(|(_, peer)| _ = peer.send(encode(&Lobby::Peer(false))));
    if room.is_empty() {
        rooms.remove(&mailbox);
    }
}
