use js_sys::Uint8Array;
use serde::Serialize;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::{Error, Event, FileEntry, Lobby, Manifest, Role, Secret, Signal};

impl From<Error> for JsValue {
    fn from(e: Error) -> Self {
        JsError::new(&e.to_string()).into()
    }
}

#[derive(Serialize, Tsify)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CoreEvent {
    Manifest {
        files: Vec<JsFile>,
        size: u64,
        chunks: u32,
    },
    Read {
        chunk: u32,
        file: u32,
        offset: u64,
        len: u32,
    },
    Write {
        chunk: u32,
        file: u32,
        offset: u64,
        #[tsify(type = "Uint8Array")]
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    Progress {
        bytes: u64,
        total: u64,
    },
    Done,
    Closed {
        reason: String,
    },
}

#[derive(Serialize, serde::Deserialize, Tsify)]
pub struct JsFile {
    pub name: String,
    pub size: u64,
    pub mime: String,
}

#[derive(Serialize, Tsify)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LobbyEvent {
    Peer {
        present: bool,
    },
    Pake {
        #[tsify(type = "Uint8Array")]
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    Sealed {
        #[tsify(type = "Uint8Array")]
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    Full,
}

#[derive(Serialize, Tsify)]
pub struct Span {
    pub file: u32,
    pub offset: u64,
    pub len: u32,
}

#[derive(Serialize, Tsify)]
pub struct ParsedLink {
    pub secret: String,
    pub mailbox: String,
    pub lobby: Option<String>,
}

fn ts<T: Tsify + Serialize>(value: T) -> Ts<T> {
    value.into_ts().unwrap()
}

impl From<Event> for CoreEvent {
    fn from(event: Event) -> Self {
        match event {
            Event::Manifest { manifest } => CoreEvent::Manifest {
                size: manifest.size(),
                chunks: manifest.chunks(),
                files: manifest
                    .files
                    .into_iter()
                    .map(|f| JsFile { name: f.name, size: f.size, mime: f.mime })
                    .collect(),
            },
            Event::Read { chunk, file, offset, len } => CoreEvent::Read { chunk, file, offset, len },
            Event::Write { chunk, file, offset, data } => CoreEvent::Write { chunk, file, offset, data },
            Event::Progress { bytes, total } => CoreEvent::Progress { bytes, total },
            Event::Done => CoreEvent::Done,
            Event::Closed { reason } => CoreEvent::Closed { reason },
        }
    }
}

#[wasm_bindgen(js_name = newSecret)]
pub fn new_secret() -> String {
    Secret::generate().encode()
}

#[wasm_bindgen(js_name = parseLink)]
pub fn parse_link(s: &str) -> Option<Ts<ParsedLink>> {
    let link = crate::Link::parse(s)?;
    Some(ts(ParsedLink { secret: link.secret.encode(), mailbox: link.secret.mailbox(), lobby: link.lobby }))
}

#[wasm_bindgen(js_name = decodeLobby)]
pub fn decode_lobby(bytes: &[u8]) -> Result<Ts<LobbyEvent>, Error> {
    Ok(ts(match postcard::from_bytes(bytes).map_err(|_| Error::Decode)? {
        Lobby::Peer(present) => LobbyEvent::Peer { present },
        Lobby::Full => LobbyEvent::Full,
        Lobby::Relay(payload) => match postcard::from_bytes(&payload).map_err(|_| Error::Decode)? {
            Signal::Pake(data) => LobbyEvent::Pake { data },
            Signal::Sealed(data) => LobbyEvent::Sealed { data },
        },
    }))
}

#[wasm_bindgen]
pub struct ManifestBuilder {
    manifest: Manifest,
}

#[wasm_bindgen]
impl ManifestBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(files: Vec<Ts<JsFile>>) -> Result<ManifestBuilder, Error> {
        let files: Vec<JsFile> =
            files.iter().map(|f| f.to_rust()).collect::<Result<_, _>>().map_err(|_| Error::Decode)?;
        let files = files.into_iter().map(|f| FileEntry { name: f.name, size: f.size, mime: f.mime }).collect();
        Ok(Self { manifest: Manifest { files, hashes: vec![] } })
    }

    pub fn chunks(&self) -> u32 {
        self.manifest.files.iter().map(FileEntry::chunks).sum()
    }

    pub fn next(&self) -> Option<Ts<Span>> {
        let (file, offset, len) = self.manifest.locate(self.manifest.hashes.len() as u32)?;
        Some(ts(Span { file, offset, len }))
    }

    pub fn push(&mut self, data: &[u8]) {
        self.manifest.hashes.push(crate::hash_chunk(data));
    }
}

#[wasm_bindgen]
pub struct Pake {
    inner: Option<crate::Handshake>,
    message: Vec<u8>,
}

#[wasm_bindgen]
impl Pake {
    #[wasm_bindgen(constructor)]
    pub fn new(secret: &str, sender: bool) -> Result<Pake, Error> {
        let secret = Secret::decode(secret).ok_or(Error::Decode)?;
        let (inner, msg) = crate::Handshake::start(&secret, if sender { Role::Sender } else { Role::Receiver });
        Ok(Self { inner: Some(inner), message: postcard::to_allocvec(&Signal::Pake(msg)).unwrap() })
    }

    pub fn message(&self) -> Vec<u8> {
        self.message.clone()
    }

    pub fn finish(&mut self, peer: &[u8]) -> Result<Keys, Error> {
        let keys = self.inner.take().ok_or(Error::Protocol)?.finish(peer)?;
        Ok(Keys { signal: keys.signal(), keys })
    }
}

#[wasm_bindgen]
pub struct Keys {
    keys: crate::Keys,
    signal: crate::Channel,
}

#[wasm_bindgen]
impl Keys {
    pub fn seal(&mut self, text: &str) -> Vec<u8> {
        postcard::to_allocvec(&Signal::Sealed(self.signal.seal(text.as_bytes()))).unwrap()
    }

    pub fn open(&self, data: &[u8]) -> Result<String, Error> {
        String::from_utf8(self.signal.open(data)?).map_err(|_| Error::Decode)
    }

    pub fn sender(&self, builder: &ManifestBuilder) -> Result<SenderSession, Error> {
        if self.keys.role() != Role::Sender || !builder.manifest.is_valid() {
            return Err(Error::Protocol);
        }
        Ok(SenderSession(crate::Sender::new(self.keys.data(), builder.manifest.clone())))
    }

    pub fn receiver(&self) -> Result<ReceiverSession, Error> {
        if self.keys.role() != Role::Receiver {
            return Err(Error::Protocol);
        }
        Ok(ReceiverSession(crate::Receiver::new(self.keys.data())))
    }
}

fn drain(mut f: impl FnMut() -> Option<Vec<u8>>) -> Vec<Uint8Array> {
    std::iter::from_fn(|| f().map(|b| Uint8Array::from(&b[..]))).collect()
}

#[wasm_bindgen]
pub struct SenderSession(crate::Sender);

#[wasm_bindgen]
impl SenderSession {
    pub fn handle(&mut self, msg: &[u8]) -> Result<(), Error> {
        self.0.handle(msg)
    }

    pub fn provide(&mut self, chunk: u32, data: &[u8]) {
        self.0.provide(chunk, data)
    }

    pub fn close(&mut self, reason: &str) {
        self.0.close(reason)
    }

    pub fn transmit(&mut self) -> Vec<Uint8Array> {
        drain(|| self.0.poll_transmit())
    }

    pub fn events(&mut self) -> Vec<Ts<CoreEvent>> {
        std::iter::from_fn(|| self.0.poll_event()).map(|e| ts(CoreEvent::from(e))).collect()
    }
}

#[wasm_bindgen]
pub struct ReceiverSession(crate::Receiver);

#[wasm_bindgen]
impl ReceiverSession {
    pub fn handle(&mut self, msg: &[u8], now: f64) -> Result<(), Error> {
        self.0.handle(msg, now as u64)
    }

    pub fn accept(&mut self, have: Option<Vec<u8>>, now: f64) {
        self.0.accept(have.as_deref(), now as u64)
    }

    pub fn written(&mut self, chunk: u32, now: f64) {
        self.0.written(chunk, now as u64)
    }

    #[wasm_bindgen(js_name = setMaxWindow)]
    pub fn set_max_window(&mut self, chunks: u32) {
        self.0.set_max_window(chunks as usize)
    }

    pub fn complete(&mut self) {
        self.0.complete()
    }

    pub fn tick(&mut self, now: f64) {
        self.0.tick(now as u64)
    }

    pub fn have(&self) -> Vec<u8> {
        self.0.have().bytes().to_vec()
    }

    pub fn root(&self) -> Option<String> {
        self.0.manifest().map(|m| data_encoding::HEXLOWER.encode(&m.root()))
    }

    pub fn close(&mut self, reason: &str) {
        self.0.close(reason)
    }

    pub fn transmit(&mut self) -> Vec<Uint8Array> {
        drain(|| self.0.poll_transmit())
    }

    pub fn events(&mut self) -> Vec<Ts<CoreEvent>> {
        std::iter::from_fn(|| self.0.poll_event()).map(|e| ts(CoreEvent::from(e))).collect()
    }
}
