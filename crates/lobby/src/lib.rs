use uploadnt_core::Lobby;

#[cfg(target_arch = "wasm32")]
mod worker;

pub const MAX_MESSAGE: usize = 64 * 1024;

pub fn encode(msg: &Lobby) -> Vec<u8> {
    postcard::to_allocvec(msg).unwrap()
}

pub fn valid_mailbox(mailbox: &str) -> bool {
    mailbox.len() == 32 && mailbox.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
