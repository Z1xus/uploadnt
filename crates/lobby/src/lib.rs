use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;
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

pub const TURN_TTL_SECS: u64 = 12 * 3600;
const STATS_DAYS: u64 = 30;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Stats(Vec<(u64, u64, u64)>);

impl Stats {
    pub fn count(&mut self, day: u64, path: &str) {
        let planb = match path {
            "direct" => false,
            "planb" => true,
            _ => return,
        };
        self.0.retain(|(d, _, _)| d + STATS_DAYS > day);
        if self.0.last().is_none_or(|(d, _, _)| *d != day) {
            self.0.push((day, 0, 0));
        }
        let (_, direct, relayed) = self.0.last_mut().unwrap();
        *if planb { relayed } else { direct } += 1;
    }

    pub fn json(&self, day: u64) -> String {
        let (direct, planb) = self
            .0
            .iter()
            .filter(|(d, _, _)| d + STATS_DAYS > day)
            .fold((0, 0), |(a, b), (_, direct, planb)| (a + direct, b + planb));
        format!("{{\"direct\":{direct},\"planb\":{planb}}}")
    }
}

// coturn use-auth-secret: the username is the expiry and the password is its hmac
pub fn coturn_credentials(urls: &str, secret: &str, now_secs: u64) -> String {
    let username = (now_secs + TURN_TTL_SECS).to_string();
    let mut mac = Hmac::<Sha1>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(username.as_bytes());
    let credential = data_encoding::BASE64.encode(&mac.finalize().into_bytes());
    let urls: Vec<&str> = urls.split(',').map(str::trim).collect();
    serde_json::json!({ "iceServers": [{ "urls": urls, "username": username, "credential": credential }] }).to_string()
}
