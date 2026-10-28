use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lobby {
    Peer(bool),
    Relay(#[serde(with = "serde_bytes")] Vec<u8>),
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    Pake(#[serde(with = "serde_bytes")] Vec<u8>),
    Sealed(#[serde(with = "serde_bytes")] Vec<u8>),
}
