use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Clone)]
pub struct SearchConsoleConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub return_uri: String,
    pub encryption_key: [u8; 32],
}

pub fn oauth_state() -> String {
    let mut value = [0_u8; 32];
    OsRng.fill_bytes(&mut value);
    URL_SAFE_NO_PAD.encode(value)
}

pub fn state_hash(state: &str) -> String {
    Sha256::digest(state.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn authorization_url(config: &SearchConsoleConfig, state: &str) -> String {
    let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth").expect("valid URL");
    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", &config.redirect_uri)
        .append_pair("response_type", "code")
        .append_pair(
            "scope",
            "https://www.googleapis.com/auth/webmasters.readonly",
        )
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("include_granted_scopes", "true")
        .append_pair("state", state);
    url.into()
}

pub fn encrypt_token(key: &[u8; 32], token: &str) -> Result<String, &'static str> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| "invalid encryption key")?;
    let mut nonce = [0_u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), token.as_bytes())
        .map_err(|_| "token encryption failed")?;
    let mut payload = nonce.to_vec();
    payload.extend(ciphertext);
    Ok(URL_SAFE_NO_PAD.encode(payload))
}

pub fn decrypt_token(key: &[u8; 32], encrypted: &str) -> Result<String, &'static str> {
    let payload = URL_SAFE_NO_PAD
        .decode(encrypted)
        .map_err(|_| "invalid encrypted token")?;
    if payload.len() <= 12 {
        return Err("invalid encrypted token");
    }
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| "invalid encryption key")?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&payload[..12]), &payload[12..])
        .map_err(|_| "token decryption failed")?;
    String::from_utf8(plaintext).map_err(|_| "invalid token encoding")
}

pub fn preferred_property(domain: &str, properties: &[String]) -> Option<String> {
    let domain_property = format!("sc-domain:{domain}");
    if properties.iter().any(|value| value == &domain_property) {
        return Some(domain_property);
    }
    let prefix = format!("https://{domain}/");
    properties.iter().find(|value| *value == &prefix).cloned()
}
