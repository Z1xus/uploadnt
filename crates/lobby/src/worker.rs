use uploadnt_core::Lobby;
use worker::*;

use crate::{MAX_MESSAGE, Stats, TURN_TTL_SECS, coturn_credentials, encode, valid_mailbox};

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let path = req.path();
    if path == "/stats" && req.method() == Method::Get {
        return tally(&env)?.fetch_with_request(req).await;
    }
    match path.strip_prefix("/m/").map(|rest| rest.split_once('/').map_or(rest, |(mailbox, _)| mailbox)) {
        Some(mailbox) if valid_mailbox(mailbox) => {
            env.durable_object("LOBBY")?.id_from_name(mailbox)?.get_stub()?.fetch_with_request(req).await
        }
        _ => Response::error("nothing here", 404),
    }
}

fn tally(env: &Env) -> Result<Stub> {
    env.durable_object("TALLY")?.id_from_name("all")?.get_stub()
}

async fn turn(env: &Env, mailbox: &str) -> Result<Response> {
    if let (Ok(urls), Ok(secret)) = (env.var("TURN_URLS"), env.secret("TURN_SECRET")) {
        let body = coturn_credentials(&urls.to_string(), &secret.to_string(), Date::now().as_millis() / 1000, mailbox);
        return json(Response::ok(body)?, "no-store");
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
    json(Response::from_bytes(res.bytes().await?)?.with_status(res.status_code()), "no-store")
}

fn json(mut res: Response, cache: &str) -> Result<Response> {
    res.headers_mut().set("Content-Type", "application/json")?;
    res.headers_mut().set("Cache-Control", cache)?;
    Ok(res)
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
            let path = req.text().await?;
            let mut stats: Stats = storage.get("days").await?.unwrap_or_default();
            stats.count(day, &path);
            storage.put("days", &stats).await?;
            return Response::empty();
        }
        let stats: Stats = storage.get("days").await?.unwrap_or_default();
        json(Response::ok(stats.json(day))?, "max-age=300")
    }
}

#[durable_object]
pub struct Room {
    state: State,
    env: Env,
}

impl Room {
    fn others(&self, ws: &WebSocket) -> Vec<WebSocket> {
        let tags = self.state.get_tags(ws);
        self.state.get_websockets().into_iter().filter(|other| self.state.get_tags(other) != tags).collect()
    }

    async fn report(&self, ws: &WebSocket, path: String) -> Result<()> {
        let fresh = ws.deserialize_attachment::<bool>()?.is_none();
        if !fresh || self.state.get_websockets().len() != 2 || !matches!(path.as_str(), "direct" | "planb") {
            return Ok(());
        }
        ws.serialize_attachment(true)?;
        let mut init = RequestInit::new();
        init.with_method(Method::Post).with_body(Some(path.into()));
        tally(&self.env)?.fetch_with_request(Request::new_with_init("https://tally/", &init)?).await?;
        Ok(())
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
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        if req.path().ends_with("/turn") {
            if self.state.get_websockets().len() != 2 {
                return Response::error("nobody to relay for", 403);
            }
            let path = req.path();
            return turn(&self.env, path.trim_start_matches("/m/").trim_end_matches("/turn")).await;
        }
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
        let bytes = match message {
            WebSocketIncomingMessage::Binary(bytes) => bytes,
            WebSocketIncomingMessage::String(path) => return self.report(&ws, path).await,
        };
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
