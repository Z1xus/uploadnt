use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use spake2::{Ed25519Group, Identity, Password, Spake2};

use crate::{Error, Secret};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Sender,
    Receiver,
}

pub struct Handshake {
    state: Spake2<Ed25519Group>,
    role: Role,
}

impl Handshake {
    pub fn start(secret: &Secret, role: Role) -> (Self, Vec<u8>) {
        let password = Password::new(secret.pake_password());
        let (a, b) = (Identity::new(b"uploadnt/sender"), Identity::new(b"uploadnt/receiver"));
        let (state, msg) = match role {
            Role::Sender => Spake2::<Ed25519Group>::start_a(&password, &a, &b),
            Role::Receiver => Spake2::<Ed25519Group>::start_b(&password, &a, &b),
        };
        (Self { state, role }, msg)
    }

    pub fn finish(self, peer: &[u8]) -> Result<Keys, Error> {
        let key = self.state.finish(peer).map_err(|_| Error::Protocol)?;
        Ok(Keys { key: key.try_into().map_err(|_| Error::Protocol)?, role: self.role })
    }
}

pub struct Keys {
    key: [u8; 32],
    role: Role,
}

impl Keys {
    pub fn role(&self) -> Role {
        self.role
    }

    pub fn data(&self) -> Channel {
        self.channel(b"uploadnt/data/s2r", b"uploadnt/data/r2s")
    }

    pub fn signal(&self) -> Channel {
        self.channel(b"uploadnt/signal/s2r", b"uploadnt/signal/r2s")
    }

    fn channel(&self, s2r: &[u8], r2s: &[u8]) -> Channel {
        let (seal, open) = match self.role {
            Role::Sender => (s2r, r2s),
            Role::Receiver => (r2s, s2r),
        };
        Channel { seal: self.cipher(seal), open: self.cipher(open), counter: 0 }
    }

    fn cipher(&self, label: &[u8]) -> ChaCha20Poly1305 {
        let mut key = [0; 32];
        Hkdf::<Sha256>::new(None, &self.key).expand(label, &mut key).unwrap();
        ChaCha20Poly1305::new(&Key::from(key))
    }
}

// each message carries its own counter because data channels are striped and arrive out of order
pub struct Channel {
    seal: ChaCha20Poly1305,
    open: ChaCha20Poly1305,
    counter: u64,
}

impl Channel {
    pub fn seal(&mut self, plain: &[u8]) -> Vec<u8> {
        let counter = self.counter.to_le_bytes();
        self.counter += 1;
        let sealed = self.seal.encrypt(&nonce(&counter), plain).unwrap();
        [&counter[..], &sealed].concat()
    }

    pub fn open(&self, msg: &[u8]) -> Result<Vec<u8>, Error> {
        let (counter, sealed) = msg.split_at_checked(8).ok_or(Error::Decrypt)?;
        self.open.decrypt(&nonce(counter), sealed).map_err(|_| Error::Decrypt)
    }
}

fn nonce(counter: &[u8]) -> Nonce {
    let mut nonce = [0; 12];
    nonce[4..].copy_from_slice(counter);
    nonce.into()
}
