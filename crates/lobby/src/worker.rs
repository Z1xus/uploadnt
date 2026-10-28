use uploadnt_core::Lobby;
use worker::*;

use crate::{MAX_MESSAGE, encode, valid_mailbox};

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let path = req.path();
    match path.strip_prefix("/m/") {
        Some(mailbox) if valid_mailbox(mailbox) => {
            env.durable_object("LOBBY")?.id_from_name(mailbox)?.get_stub()?.fetch_with_request(req).await
        }
        _ => Response::error("nothing here", 404),
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
