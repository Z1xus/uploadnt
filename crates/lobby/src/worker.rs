use uploadnt_core::Lobby;
use worker::*;

use crate::{MAX_MESSAGE, Stats, TURN_TTL_SECS, coturn_credentials, encode, valid_mailbox};

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let path = req.path();
    if path == "/turn/credentials" {
        return turn(&env).await;
    }
    if path == "/stats" {
        return env.durable_object("TALLY")?.id_from_name("all")?.get_stub()?.fetch_with_request(req).await;
    }
    match path.strip_prefix("/m/") {
        Some(mailbox) if valid_mailbox(mailbox) => {
            env.durable_object("LOBBY")?.id_from_name(mailbox)?.get_stub()?.fetch_with_request(req).await
        }
        _ => Response::error("nothing here", 404),
    }
}

async fn turn(env: &Env) -> Result<Response> {
    if let (Ok(urls), Ok(secret)) = (env.var("TURN_URLS"), env.secret("TURN_SECRET")) {
        let body = coturn_credentials(&urls.to_string(), &secret.to_string(), Date::now().as_millis() / 1000);
        let mut out = Response::ok(body)?;
        out.headers_mut().set("Content-Type", "application/json")?;
        out.headers_mut().set("Cache-Control", "no-store")?;
        return Ok(out);
    }
    let (Ok(id), Ok(token)) = (env.var("TURN_KEY_ID"), env.secret("TURN_KEY_API_TOKEN")) else {
        return Response::error("no plan b here", 404);
    };
    let url = format!("https://rtc.live.cloudflare.com/v1/turn/keys/{id}/credentials/generate-ice-servers");
    let headers = Headers::new();
    headers.set("Authorization", &format!("Bearer {token}"))?;
    headers.set("Content-Type", "application/json")?;
    let mut init = RequestInit::new();
    init.with_method(Method::Post).with_headers(headers).with_body(Some(format!("{{\"ttl\":{TURN_TTL_SECS}}}").into()));
    let mut res = Fetch::Request(Request::new_with_init(&url, &init)?).send().await?;
    let mut out = Response::from_bytes(res.bytes().await?)?.with_status(res.status_code());
    out.headers_mut().set("Content-Type", "application/json")?;
    out.headers_mut().set("Cache-Control", "no-store")?;
    Ok(out)
}

#[durable_object]
pub struct Tally {
    state: State,
}

impl DurableObject for Tally {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, mut req: Request) -> Result<Response> {
        let day = Date::now().as_millis() / 86_400_000;
        let storage = self.state.storage();
        if req.method() == Method::Post {
            if req.headers().get("Content-Length")?.and_then(|n| n.parse::<usize>().ok()).is_none_or(|n| n > 8) {
                return Response::error("too big", 413);
            }
            let path = req.text().await?;
            let mut stats: Stats = storage.get("days").await?.unwrap_or_default();
            stats.count(day, &path);
            storage.put("days", &stats).await?;
            return Response::empty();
        }
        let stats: Stats = storage.get("days").await?.unwrap_or_default();
        let mut res = Response::ok(stats.json(day))?;
        res.headers_mut().set("Content-Type", "application/json")?;
        res.headers_mut().set("Cache-Control", "max-age=300")?;
        Ok(res)
    }
}

#[durable_object]
pub struct Room {
    state: State,
}

impl Room {
    fn others(&self, ws: &WebSocket) -> Vec<WebSocket> {
        let tags = self.state.get_tags(ws);
        self.state.get_websockets().into_iter().filter(|other| self.state.get_tags(other) != tags).collect()
    }

    fn leave(&self, ws: &WebSocket) -> Result<()> {
        for other in self.others(ws) {
            other.send_with_bytes(encode(&Lobby::Peer(false)))?;
        }
        _ = ws.close(Some(1000), Some("bye"));
        Ok(())
    }
}

impl DurableObject for Room {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        if req.headers().get("Upgrade")?.as_deref() != Some("websocket") {
            return Response::error("websockets only", 426);
        }
        let pair = WebSocketPair::new()?;
        let taken: Vec<String> = self.state.get_websockets().iter().flat_map(|ws| self.state.get_tags(ws)).collect();
        match ["a", "b"].into_iter().find(|tag| !taken.iter().any(|t| t == tag)) {
            Some(tag) => {
                self.state.accept_websocket_with_tags(&pair.server, &[tag]);
                if taken.len() == 1 {
                    for ws in self.state.get_websockets() {
                        ws.send_with_bytes(encode(&Lobby::Peer(true)))?;
                    }
                }
            }
            None => {
                pair.server.accept()?;
                pair.server.send_with_bytes(encode(&Lobby::Full))?;
                pair.server.close(Some(1008), Some("full"))?;
            }
        }
        Response::from_websocket(pair.client)
    }

    async fn websocket_message(&self, ws: WebSocket, message: WebSocketIncomingMessage) -> Result<()> {
        let WebSocketIncomingMessage::Binary(bytes) = message else { return Ok(()) };
        if bytes.len() > MAX_MESSAGE {
            return ws.close(Some(1009), Some("too big"));
        }
        let relay = encode(&Lobby::Relay(bytes));
        for other in self.others(&ws) {
            other.send_with_bytes(&relay)?;
        }
        Ok(())
    }

    async fn websocket_close(&self, ws: WebSocket, _code: usize, _reason: String, _clean: bool) -> Result<()> {
        self.leave(&ws)
    }

    async fn websocket_error(&self, ws: WebSocket, _error: Error) -> Result<()> {
        self.leave(&ws)
    }
}
