mod bitmap;
mod crypto;
mod frame;
mod link;
mod lobby;
mod manifest;
mod session;
#[cfg(feature = "wasm")]
mod wasm;

pub use bitmap::Bitmap;
pub use crypto::{Channel, Handshake, Keys, Role};
pub use frame::{Frame, VERSION};
pub use link::{Link, Secret};
pub use lobby::{Lobby, Signal};
pub use manifest::{CHUNK, FileEntry, Manifest, PIECE, hash_chunk};
pub use session::{Event, Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Decrypt,
    Decode,
    Version,
    Protocol,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Error::Decrypt => "decrypt",
            Error::Decode => "decode",
            Error::Version => "version",
            Error::Protocol => "protocol",
        })
    }
}

impl std::error::Error for Error {}
