use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Request, State};
use axum::http::{HeaderName, HeaderValue, StatusCode, header};
use axum::middleware::{map_request_with_state, map_response};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use lobby::{MAX_MESSAGE, Stats, TURNS_PER_ROOM, coturn_credentials, encode, valid_mailbox};
use tokio::sync::mpsc::{Sender, channel};
use tokio::time::timeout;
use tower_http::services::{ServeDir, ServeFile};
use uploadnt_core::Lobby;

#[derive(Default)]
struct Room {
    peers: Vec<(u64, Sender<Vec<u8>>)>,
    turns: u32,
}

#[derive(Default)]
struct Lobbies {
    rooms: Mutex<HashMap<String, Room>>,
    stats: Mutex<Stats>,
}

type App = Arc<Lobbies>;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
const PING: Duration = Duration::from_secs(20);
const DEAD: Duration = Duration::from_secs(60);
// signalling needs a handful of messages, a backlog this long means the peer stopped reading
const QUEUE: usize = 64;
const MAX_ROOMS: usize = 10_000;
// a handshake takes a few dozen messages, so a socket sending this many in a window is flooding
const MESSAGES_PER_WINDOW: u32 = 500;
const WINDOW: Duration = Duration::from_secs(10);

#[tokio::main]
pub async fn main() {
    let site = std::env::var("SITE").unwrap_or_else(|_| "web/build".into());
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "0.0.0.0:8787".into());
    let headers = Arc::new(site_headers(&format!("{site}/_headers")));
    let pages = Router::new()
        .fallback_service(ServeDir::new(&site).fallback(ServeFile::new(format!("{site}/index.html"))))
        .layer(map_request_with_state(Arc::<str>::from(site.as_str()), html_page));
    let app = Router::new()
        .route("/m/{mailbox}", get(join))
        .route("/m/{mailbox}/turn", get(turn))
        .route("/stats", get(stats))
        .with_state(App::default())
        .fallback_service(pages)
        .layer(map_response(move |mut res: Response| {
            let headers = headers.clone();
            async move {
                res.headers_mut().extend(headers.iter().cloned());
                res
            }
        }));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("lobby on {addr}, serving {site}");
    axum::serve(listener, app).await.unwrap();
}

fn site_headers(path: &str) -> Vec<(HeaderName, HeaderValue)> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    text.lines()
        .filter_map(|line| line.strip_prefix("  ")?.split_once(": "))
        .filter_map(|(name, value)| Some((name.parse().ok()?, value.parse().ok()?)))
        .collect()
}

fn now() -> Duration {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap()
}

fn today() -> u64 {
    now().as_secs() / 86400
}

fn paired(app: &App, mailbox: &str) -> bool {
    app.rooms.lock().unwrap().get(mailbox).is_some_and(|room| room.peers.len() == 2)
}

async fn turn(Path(mailbox): Path<String>, State(app): State<App>) -> Response {
    let secret = std::env::var("TURN_SECRET").ok().or_else(|| {
        let file = std::fs::read_to_string(std::env::var("TURN_SECRET_FILE").ok()?).ok()?;
        Some(file.trim().to_owned())
    });
    let (Ok(urls), Some(secret)) = (std::env::var("TURN_URLS"), secret) else {
        return (StatusCode::NOT_FOUND, "no plan b here").into_response();
    };
    {
        let mut rooms = app.rooms.lock().unwrap();
        let Some(room) = rooms.get_mut(&mailbox).filter(|room| room.peers.len() == 2) else {
            return (StatusCode::FORBIDDEN, "nobody to relay for").into_response();
        };
        room.turns += 1;
        if room.turns > TURNS_PER_ROOM {
            return (StatusCode::TOO_MANY_REQUESTS, "enough relay logins for this transfer").into_response();
        }
    }
    let body = coturn_credentials(&urls, &secret, now().as_secs(), &mailbox);
    ([(header::CONTENT_TYPE, "application/json"), (header::CACHE_CONTROL, "no-store")], body).into_response()
}

// a path without a dot cannot climb out of the site folder
async fn html_page(State(site): State<Arc<str>>, mut req: Request) -> Request {
    let path = req.uri().path();
    if path != "/" && !path.contains('.') && std::path::Path::new(&format!("{site}{path}.html")).is_file() {
        *req.uri_mut() = format!("{path}.html").parse().unwrap();
    }
    req
}

async fn stats(State(app): State<App>) -> Response {
    ([(header::CONTENT_TYPE, "application/json")], app.stats.lock().unwrap().json(today())).into_response()
}

async fn join(Path(mailbox): Path<String>, State(app): State<App>, ws: WebSocketUpgrade) -> Response {
    ws.max_message_size(MAX_MESSAGE).on_upgrade(move |socket| run(socket, mailbox, app))
}

async fn run(mut socket: WebSocket, mailbox: String, app: App) {
    if !valid_mailbox(&mailbox) {
        return;
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, mut rx) = channel(QUEUE);
    let joined = {
        let mut rooms = app.rooms.lock().unwrap();
        if rooms.len() >= MAX_ROOMS && !rooms.contains_key(&mailbox) {
            return;
        }
        let room = rooms.entry(mailbox.clone()).or_default();
        if room.peers.len() < 2 {
            room.peers.push((id, tx));
            if room.peers.len() == 2 {
                room.peers.iter().for_each(|(_, peer)| _ = peer.try_send(encode(&Lobby::Peer(true))));
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
    let mut reported = false;
    let (mut window, mut messages) = (Instant::now(), 0);
    loop {
        let out = tokio::select! {
            msg = socket.recv() => {
                seen = Instant::now();
                if window.elapsed() > WINDOW {
                    (window, messages) = (seen, 0);
                }
                messages += 1;
                if messages > MESSAGES_PER_WINDOW {
                    break;
                }
                match msg {
                    Some(Ok(Message::Binary(bytes))) => {
                        let relay = encode(&Lobby::Relay(bytes.to_vec()));
                        let rooms = app.rooms.lock().unwrap();
                        rooms[&mailbox].peers.iter().filter(|(peer, _)| *peer != id).for_each(|(_, peer)| _ = peer.try_send(relay.clone()));
                    }
                    Some(Ok(Message::Text(path))) if !reported && paired(&app, &mailbox) => {
                        reported = true;
                        app.stats.lock().unwrap().count(today(), &path);
                    }
                    Some(Ok(_)) => {}
                    _ => break,
                }
                continue;
            }
            Some(out) = rx.recv() => Message::Binary(out.into()),
            _ = ping.tick() => {
                if seen.elapsed() > DEAD {
                    break;
                }
                Message::Ping(Default::default())
            }
        };
        if !matches!(timeout(DEAD, socket.send(out)).await, Ok(Ok(()))) {
            break;
        }
    }

    let mut rooms = app.rooms.lock().unwrap();
    let room = rooms.get_mut(&mailbox).unwrap();
    room.peers.retain(|(peer, _)| *peer != id);
    room.peers.iter().for_each(|(_, peer)| _ = peer.try_send(encode(&Lobby::Peer(false))));
    if room.peers.is_empty() {
        rooms.remove(&mailbox);
    }
}
