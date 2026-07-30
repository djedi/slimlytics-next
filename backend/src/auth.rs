use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::{rngs::OsRng as TokenRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("password operation failed")]
    Password,
    #[error("invalid token")]
    Token(#[from] jsonwebtoken::errors::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|value| value.to_string())
        .map_err(|_| AuthError::Password)
}

pub fn verify_password(password: &str, encoded: &str) -> Result<bool, AuthError> {
    let parsed = PasswordHash::new(encoded).map_err(|_| AuthError::Password)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn issue_token(user_id: Uuid, secret: &str, ttl_seconds: i64) -> Result<String, AuthError> {
    let now = Utc::now().timestamp();
    Ok(encode(
        &Header::default(),
        &Claims {
            sub: user_id,
            iat: now as usize,
            exp: (now + ttl_seconds) as usize,
        },
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AuthError> {
    Ok(decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?
    .claims)
}

pub fn generate_api_token() -> String {
    let mut bytes = [0_u8; 32];
    TokenRng.fill_bytes(&mut bytes);
    format!("slyt_{}", URL_SAFE_NO_PAD.encode(bytes))
}

pub fn hash_api_token(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}
