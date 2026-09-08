//! JWT and password auth for Open IEM Platform.
//!
//! - Access tokens: Ed25519-signed JWT (15-minute lifetime)
//! - Refresh tokens: random 256-bit opaque token (12-hour lifetime)
//! - Passwords: Argon2id with auto-generated salt (argon2 0.6 / password-hash 0.6)

use crate::error::ApiError;
use control_protocol::Role;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT issuer string.
pub const JWT_ISSUER: &str = "open-iem-platform";
/// JWT audience string.
pub const JWT_AUDIENCE: &str = "open-iem-clients";
/// Access token lifetime in seconds (15 minutes).
pub const ACCESS_TOKEN_TTL_S: u64 = 15 * 60;
/// Refresh token lifetime in seconds (12 hours).
pub const REFRESH_TOKEN_TTL_S: u64 = 12 * 60 * 60;

/// Claims embedded in the access JWT.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JwtClaims {
    /// Subject — username.
    pub sub: String,
    /// Role.
    pub role: Role,
    /// JWT ID (unique per token — for revocation).
    pub jti: String,
    /// Issuer.
    pub iss: String,
    /// Audience.
    pub aud: String,
    /// Issued-at (Unix seconds).
    pub iat: u64,
    /// Expiry (Unix seconds).
    pub exp: u64,
}

/// Keys used for JWT sign/verify. Ed25519 keypair.
/// In production these are loaded from disk/env — never hardcoded.
pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    /// Build from raw Ed25519 PEM bytes.
    ///
    /// # Errors
    /// Returns `ApiError::Internal` if PEM is invalid.
    pub fn from_ed_pem(private_pem: &[u8], public_pem: &[u8]) -> Result<Self, ApiError> {
        Ok(Self {
            encoding: EncodingKey::from_ed_pem(private_pem)
                .map_err(|e| ApiError::Internal(e.to_string()))?,
            decoding: DecodingKey::from_ed_pem(public_pem)
                .map_err(|e| ApiError::Internal(e.to_string()))?,
        })
    }

    /// Issue an access token for the given user and role.
    ///
    /// # Errors
    /// Returns `ApiError::Internal` if signing fails.
    pub fn issue(&self, username: &str, role: Role, jti: &str) -> Result<String, ApiError> {
        let now = unix_now();
        let claims = JwtClaims {
            sub: username.to_owned(),
            role,
            jti: jti.to_owned(),
            iss: JWT_ISSUER.to_owned(),
            aud: JWT_AUDIENCE.to_owned(),
            iat: now,
            exp: now + ACCESS_TOKEN_TTL_S,
        };
        encode(&Header::new(Algorithm::EdDSA), &claims, &self.encoding)
            .map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Verify and decode an access token.
    ///
    /// # Errors
    /// Returns `ApiError::Unauthorized` on any validation failure.
    pub fn verify(&self, token: &str) -> Result<JwtClaims, ApiError> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[JWT_ISSUER]);
        validation.set_audience(&[JWT_AUDIENCE]);
        validation.validate_exp = true;
        validation.validate_nbf = false;
        let data = decode::<JwtClaims>(token, &self.decoding, &validation)
            .map_err(|_| ApiError::Unauthorized("invalid or expired token"))?;
        Ok(data.claims)
    }
}

/// Generate a cryptographically random 256-bit refresh token (hex-encoded, 64 chars).
#[must_use]
pub fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes_to_hex(&bytes)
}

/// Hash a plaintext password with Argon2id.
///
/// Uses argon2 0.6 API: `hash_password(password)` generates a random salt via getrandom.
/// Returns a PHC-format string.
///
/// # Errors
/// Returns `ApiError::Internal` on hashing failure.
pub fn hash_password(password: &str) -> Result<String, ApiError> {
    use argon2::{password_hash::PasswordHasher, Argon2};
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|h| h.to_string())
        .map_err(|e| ApiError::Internal(e.to_string()))
}

/// Verify a plaintext password against an Argon2id PHC hash string.
///
/// # Errors
/// Returns `ApiError::Unauthorized` on mismatch.
pub fn verify_password(password: &str, hash: &str) -> Result<(), ApiError> {
    use argon2::{
        password_hash::{phc::PasswordHash, PasswordVerifier},
        Argon2,
    };
    let parsed =
        PasswordHash::new(hash).map_err(|_| ApiError::Unauthorized("invalid password hash"))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| ApiError::Unauthorized("invalid credentials"))
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            use std::fmt::Write;
            let _ = write!(s, "{b:02x}");
            s
        })
}

/// Hash a refresh token for safe DB storage using SHA-256.
///
/// The raw token is 256-bit random (never guessable).
/// SHA-256 hashing means a DB dump exposes only hashes, not usable tokens.
/// Returns a lowercase hex-encoded 64-char SHA-256 digest.
#[must_use]
pub fn token_to_storage_key(raw_token: &str) -> String {
    let digest = Sha256::digest(raw_token.as_bytes());
    bytes_to_hex(&digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_and_verify_roundtrip() {
        let hash = hash_password("correct-horse-battery-staple").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password("correct-horse-battery-staple", &hash).is_ok());
    }

    #[test]
    fn wrong_password_is_rejected() {
        let hash = hash_password("secret123").unwrap();
        assert!(verify_password("wrong", &hash).is_err());
    }

    #[test]
    fn refresh_token_is_64_hex_chars() {
        let t = generate_refresh_token();
        assert_eq!(t.len(), 64);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
