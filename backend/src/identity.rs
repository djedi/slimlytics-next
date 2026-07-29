use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Datelike, Timelike, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnonymousIds {
    pub visitor_id: String,
    pub session_id: String,
}

fn digest(secret: &[u8], value: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).expect("HMAC supports any key length");
    mac.update(value.as_bytes());
    URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
}

/// Derives non-reversible IDs. Visitor salt rotates daily and session salt every 30 minutes.
pub fn derive_ids(
    secret: &[u8],
    site: &str,
    ip: &str,
    user_agent: &str,
    at: DateTime<Utc>,
) -> AnonymousIds {
    let day = format!("{:04}-{:02}-{:02}", at.year(), at.month(), at.day());
    let fingerprint = format!("{site}|{ip}|{user_agent}");
    let visitor_id = digest(secret, &format!("visitor|{day}|{fingerprint}"));
    let bucket = at.hour() * 2 + at.minute() / 30;
    let session_id = digest(secret, &format!("session|{day}|{bucket}|{fingerprint}"));
    AnonymousIds {
        visitor_id,
        session_id,
    }
}
