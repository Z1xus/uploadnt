use data_encoding::BASE32_NOPAD;
use hkdf::Hkdf;
use sha2::Sha256;

#[derive(Clone, PartialEq, Eq)]
pub struct Secret([u8; 16]);

impl Secret {
    pub fn generate() -> Self {
        let mut bytes = [0; 16];
        getrandom::getrandom(&mut bytes).expect("no system randomness");
        Self(bytes)
    }

    pub fn encode(&self) -> String {
        BASE32_NOPAD.encode(&self.0).to_ascii_lowercase()
    }

    pub fn decode(s: &str) -> Option<Self> {
        let bytes = BASE32_NOPAD.decode(s.trim().to_ascii_uppercase().as_bytes()).ok()?;
        Some(Self(bytes.try_into().ok()?))
    }

    pub fn mailbox(&self) -> String {
        let mut out = [0; 16];
        self.expand(b"uploadnt/mailbox", &mut out);
        data_encoding::HEXLOWER.encode(&out)
    }

    pub(crate) fn pake_password(&self) -> [u8; 32] {
        let mut out = [0; 32];
        self.expand(b"uploadnt/pake", &mut out);
        out
    }

    fn expand(&self, label: &[u8], out: &mut [u8]) {
        Hkdf::<Sha256>::new(None, &self.0).expand(label, out).unwrap();
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Link {
    pub secret: Secret,
    pub lobby: Option<String>,
}

impl Link {
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let s = s.strip_prefix("nope://").unwrap_or(s);
        let s = s.rsplit_once('#').map_or(s, |(_, frag)| frag);
        let (secret, lobby) = match s.split_once('@') {
            Some((secret, lobby)) if !lobby.is_empty() => (secret, Some(lobby.to_owned())),
            _ => (s, None),
        };
        Some(Self { secret: Secret::decode(secret)?, lobby })
    }

    pub fn fragment(&self) -> String {
        match &self.lobby {
            Some(lobby) => format!("{}@{lobby}", self.secret.encode()),
            None => self.secret.encode(),
        }
    }
}
