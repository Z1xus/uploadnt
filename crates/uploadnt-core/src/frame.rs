use serde::{Deserialize, Serialize};

use crate::Manifest;

pub const VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Frame {
    Hello {
        version: u16,
    },
    Manifest(Manifest),
    Have(#[serde(with = "serde_bytes")] Vec<u8>),
    Want(Vec<u32>),
    Piece {
        chunk: u32,
        offset: u32,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    Ack(u32),
    Done([u8; 32]),
    Bye(String),
}
