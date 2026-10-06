//! Secure QR musician onboarding endpoints.
//!
//! RBAC policy: QR status, activation, rotation, and deactivation require
//! `Engineer` minimum role; `Admin` inherits access through role hierarchy.
//! QR exchange is public by design and grants only `Musician` credentials.
#![allow(
    missing_docs,
    clippy::format_collect,
    clippy::missing_errors_doc,
    clippy::unused_async
)]
use crate::{
    auth::{generate_refresh_token, hash_password, token_to_storage_key, JwtClaims},
    error::ApiError,
    middleware::require_min_role,
    routes::auth::refresh_cookie,
    state::AppState,
};
use axum::{
    extract::{connect_info::ConnectInfo, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use control_protocol::Role;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    net::SocketAddr,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const QR_TTL: u64 = 10 * 60;
const QR_SESSION_TTL_DEFAULT: u64 = 4 * 60 * 60;
const QR_SESSION_TTL_MAX: u64 = 24 * 60 * 60;
/// Maximum lifetime of one active generation in test mode.
const QR_ROTATION_INTERVAL: u64 = 60 * 60;

fn qr_session_ttl() -> u64 {
    std::env::var("OPENIEM_QR_SESSION_TTL_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| (60..=QR_SESSION_TTL_MAX).contains(value))
        .unwrap_or(QR_SESSION_TTL_DEFAULT)
}
const MAX_NAME: usize = 80;
const QR_SECRET_HEX_LEN: usize = 64;
const INSTRUMENTS: &[&str] = &[
    "vocals",
    "guitar",
    "bass",
    "drums",
    "keys",
    "acoustic-guitar",
    "brass",
    "strings",
];

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn qr_secret() -> String {
    let mut b = [0u8; 32];
    rand::rng().fill(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn validate_secret(secret: &str) -> Result<(), ApiError> {
    if secret.len() != QR_SECRET_HEX_LEN || !secret.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ApiError::BadRequest(
            "qr_secret format is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn validate_account(username: &str, password: &str) -> Result<(), ApiError> {
    if username.len() < 3
        || username.len() > 64
        || !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        return Err(ApiError::BadRequest(
            "username format is invalid".to_owned(),
        ));
    }
    if password.chars().count() < 12
        || password.chars().count() > 128
        || password.chars().any(char::is_control)
    {
        return Err(ApiError::BadRequest(
            "password must contain 12-128 characters and no control characters".to_owned(),
        ));
    }
    Ok(())
}

fn validate(name: &str, instrument: &str) -> Result<(), ApiError> {
    if name.trim().is_empty()
        || name.chars().count() > MAX_NAME
        || name.chars().any(char::is_control)
    {
        return Err(ApiError::BadRequest(
            "display_name must contain 1-80 Unicode characters and no control characters"
                .to_owned(),
        ));
    }
    if !INSTRUMENTS.contains(&instrument) {
        return Err(ApiError::BadRequest(
            "instrument_id is not in controlled catalog".to_owned(),
        ));
    }
    Ok(())
}
fn unix_hash(secret: &str) -> String {
    let mut h = Sha256::new();
    h.update(secret.as_bytes());
    h.finalize().iter().map(|x| format!("{x:02x}")).collect()
}

#[derive(Deserialize)]
pub struct QrConfigure {
    pub expires_in_seconds: Option<u64>,
    pub band_id: Option<i64>,
}
#[derive(Serialize)]
pub struct QrStatus {
    pub active: bool,
    pub expires_at: u64,
    pub remaining_uses: u64,
    pub generation: u64,
    pub band_id: Option<i64>,
}
#[derive(Deserialize)]
pub struct Exchange {
    pub qr_secret: String,
    pub username: String,
    pub password: String,
    pub display_name: String,
    pub instrument_id: String,
}
#[derive(Serialize)]
pub struct ExchangeResponse {
    pub access_token: String,
    pub role: Role,
}

/// Passwordless invitation-only session bootstrap. This never creates a
/// credential-bearing account and derives band scope from the QR generation.
#[derive(Deserialize)]
pub struct SessionBootstrap {
    pub qr_secret: String,
    pub display_name: String,
    pub instrument_id: String,
}

pub async fn status(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<Json<QrStatus>, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let current = now();
    let _ = state
        .db
        .expire_qr_generation_if_due(current, QR_ROTATION_INTERVAL)?;
    let (active, expires, remaining, generation, band_id) = state.db.qr_status(current)?;
    Ok(Json(QrStatus {
        active,
        expires_at: expires,
        remaining_uses: remaining,
        generation,
        band_id,
    }))
}
async fn configure(
    state: AppState,
    claims: JwtClaims,
    body: QrConfigure,
    action: &'static str,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let secret = qr_secret();
    let lifetime = body.expires_in_seconds.unwrap_or(QR_TTL).min(QR_TTL);
    if lifetime == 0 {
        return Err(ApiError::BadRequest(
            "expires_in_seconds must be greater than zero".to_owned(),
        ));
    }
    let expires = now() + lifetime;
    state.db.configure_qr_audited(
        &unix_hash(&secret),
        expires,
        body.band_id,
        claims.user_id,
        action,
    )?;
    let session_base = std::env::var("OPENIEM_SESSION_PUBLIC_BASE")
        .unwrap_or_else(|_| "http://localhost:5173".to_owned());
    let session_url = format!(
        "{}/#invitation={secret}",
        session_base.trim_end_matches('/')
    );
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "session_url": session_url,
            "expires_at": expires,
            "band_id": body.band_id
        })),
    ))
}
pub async fn activate(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<QrConfigure>,
) -> Result<impl IntoResponse, ApiError> {
    configure(state, claims, body, "ACTIVATE").await
}

pub async fn rotate(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Json(body): Json<QrConfigure>,
) -> Result<impl IntoResponse, ApiError> {
    configure(state, claims, body, "ROTATE").await
}
pub async fn deactivate(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<StatusCode, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    state.db.deactivate_qr_audited(claims.user_id)?;
    Ok(StatusCode::NO_CONTENT)
}
pub async fn session_bootstrap(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(body): Json<SessionBootstrap>,
) -> Result<impl IntoResponse, ApiError> {
    if !state.qr_exchange_limiter.allow(peer.ip(), Instant::now()) {
        return Err(ApiError::TooManyRequests);
    }
    validate_secret(&body.qr_secret)?;
    validate(&body.display_name, &body.instrument_id)?;
    let current = now();
    state
        .db
        .expire_qr_generation_if_due(current, QR_ROTATION_INTERVAL)?;
    let (_, expires, _, generation, _) = state.db.qr_status(current)?;
    if expires <= current {
        return Err(ApiError::Unauthorized("invalid or expired QR invitation"));
    }
    let raw_refresh = generate_refresh_token();
    let jti = Uuid::new_v4().to_string();
    // QR secret remains usable only until `expires`; successful session receives
    // its own configurable lifetime and does not inherit the short QR lifetime.
    let refresh_exp = current.saturating_add(qr_session_ttl());
    let access_exp = current.saturating_add(crate::auth::ACCESS_TOKEN_TTL_S);
    let (user_id, session_id, username) = match state.db.exchange_qr_session(
        &unix_hash(&body.qr_secret),
        &body.display_name,
        &body.instrument_id,
        current,
        &token_to_storage_key(&raw_refresh),
        refresh_exp,
        &Uuid::new_v4().to_string(),
        &jti,
        access_exp,
    ) {
        Ok(value) => value,
        Err(error) => {
            let _ = state
                .db
                .record_qr_audit(None, "EXCHANGE", generation, false);
            return Err(error);
        }
    };
    let access = match state.jwt.issue_with_session(
        &username,
        user_id,
        Role::Musician,
        &jti,
        Some(session_id),
    ) {
        Ok(value) => value,
        Err(error) => {
            state.db.cleanup_qr_exchange(
                &token_to_storage_key(&raw_refresh),
                &jti,
                user_id,
                session_id,
            )?;
            return Err(error);
        }
    };
    state
        .db
        .record_qr_audit(None, "EXCHANGE", generation, true)?;
    Ok((
        StatusCode::CREATED,
        [(
            header::SET_COOKIE,
            refresh_cookie(&raw_refresh, refresh_exp.saturating_sub(current)),
        )],
        Json(ExchangeResponse {
            access_token: access,
            role: Role::Musician,
        }),
    ))
}

pub async fn exchange(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(body): Json<Exchange>,
) -> Result<impl IntoResponse, ApiError> {
    if !state.qr_exchange_limiter.allow(peer.ip(), Instant::now()) {
        return Err(ApiError::TooManyRequests);
    }
    let current = now();
    let _ = state
        .db
        .expire_qr_generation_if_due(current, QR_ROTATION_INTERVAL)?;
    validate_secret(&body.qr_secret)?;
    validate_account(&body.username, &body.password)?;
    validate(&body.display_name, &body.instrument_id)?;
    let password_hash = hash_password(&body.password)?;
    let raw_refresh = generate_refresh_token();
    let jti = Uuid::new_v4().to_string();
    let now = now();
    let (user_id, session_id, username) = match state.db.exchange_qr_account(
        &unix_hash(&body.qr_secret),
        &body.username,
        &password_hash,
        &body.display_name,
        &body.instrument_id,
        now,
        &token_to_storage_key(&raw_refresh),
        now + crate::auth::REFRESH_TOKEN_TTL_S,
        &Uuid::new_v4().to_string(),
        &jti,
        now + crate::auth::ACCESS_TOKEN_TTL_S,
    ) {
        Ok(value) => value,
        Err(error) => {
            let generation = state
                .db
                .qr_status(now)
                .map(|status| status.3)
                .unwrap_or_default();
            let _ = state
                .db
                .record_qr_audit(None, "EXCHANGE", generation, false);
            return Err(error);
        }
    };
    let role = Role::Musician;
    let access =
        match state
            .jwt
            .issue_with_session(&username, user_id, role, &jti, Some(session_id))
        {
            Ok(access) => access,
            Err(error) => {
                // DB exchange commits before JWT signing. Remove all newly-created rows and
                // restore invitation capacity when signing fails, otherwise QR use leaks.
                state.db.cleanup_qr_exchange(
                    &token_to_storage_key(&raw_refresh),
                    &jti,
                    user_id,
                    session_id,
                )?;
                return Err(error);
            }
        };
    let generation = state.db.qr_status(now)?.3;
    if let Err(error) = state.db.record_qr_audit(None, "EXCHANGE", generation, true) {
        state.db.cleanup_qr_exchange(
            &token_to_storage_key(&raw_refresh),
            &jti,
            user_id,
            session_id,
        )?;
        return Err(error);
    }
    Ok((
        StatusCode::CREATED,
        [(
            header::SET_COOKIE,
            refresh_cookie(&raw_refresh, crate::auth::REFRESH_TOKEN_TTL_S),
        )],
        Json(ExchangeResponse {
            access_token: access,
            role,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::JwtClaims;
    use control_protocol::Role;

    fn claims(role: Role) -> JwtClaims {
        JwtClaims {
            sub: "test".to_owned(),
            user_id: 1,
            role,
            jti: "jti".to_owned(),
            session_id: None,
            iss: crate::auth::JWT_ISSUER.to_owned(),
            aud: crate::auth::JWT_AUDIENCE.to_owned(),
            iat: 1,
            exp: u64::MAX,
        }
    }

    #[test]
    fn account_validation_enforces_username_and_password_policy() {
        assert!(validate_account("ana_1", "a-secure-password").is_ok());
        assert!(validate_account("ab", "a-secure-password").is_err());
        assert!(validate_account("ana", "short").is_err());
        assert!(validate_account("ana!", "a-secure-password").is_err());
    }

    #[test]
    fn validation_accepts_unicode_limit_and_rejects_bad_input() {
        assert!(validate(&"é".repeat(MAX_NAME), "guitar").is_ok());
        assert!(validate(&"é".repeat(MAX_NAME + 1), "guitar").is_err());
        assert!(validate("name\n", "guitar").is_err());
        assert!(validate("name", "mandolin").is_err());
        assert!(validate("   ", "guitar").is_err());
    }

    #[test]
    fn qr_management_requires_engineer_for_all_operations() {
        assert!(require_min_role(&claims(Role::Musician), Role::Engineer).is_err());
        for role in [Role::Engineer, Role::Admin] {
            assert!(require_min_role(&claims(role), Role::Engineer).is_ok());
        }
    }

    #[test]
    fn qr_secret_hash_is_one_way_and_deterministic() {
        let secret = qr_secret();
        assert_eq!(unix_hash(&secret), unix_hash(&secret));
        assert_ne!(secret, unix_hash(&secret));
        assert_eq!(unix_hash(&secret).len(), 64);
    }

    #[test]
    fn ttl_is_capped_at_ten_minutes() {
        assert_eq!(QR_TTL, 600);
    }

    #[tokio::test]
    async fn exchange_validation_runs_before_database_use() {
        let result = validate("bad\u{0000}name", "guitar");
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn expiry_replay_and_rotation_rules_are_explicit() {
        let db = crate::db::Db::open_in_memory().expect("db");
        let first = qr_secret();
        db.configure_qr(Some(&unix_hash(&first)), 100, true)
            .expect("activate");
        let expired = db.exchange_qr(
            &unix_hash(&first),
            "Ana",
            "guitar",
            100,
            "r1",
            200,
            "f1",
            "j1",
            200,
        );
        assert!(expired.is_err());
        db.configure_qr(Some(&unix_hash(&first)), 1_000, true)
            .expect("rotate");
        let ok = db
            .exchange_qr(
                &unix_hash(&first),
                "Ana",
                "guitar",
                101,
                "r2",
                200,
                "f2",
                "j2",
                200,
            )
            .expect("first use");
        assert!(ok.2.starts_with("musician-"));
        let replay = db.exchange_qr(
            &unix_hash(&first),
            "Ana",
            "guitar",
            101,
            "r3",
            200,
            "f3",
            "j3",
            200,
        );
        assert!(replay.is_err());
    }

    #[test]
    fn status_and_deactivation_expire_active_qr() {
        let db = crate::db::Db::open_in_memory().expect("db");
        let secret = qr_secret();
        db.configure_qr(Some(&unix_hash(&secret)), 200, true)
            .expect("activate");
        assert!(db.qr_status(100).expect("status").0);
        db.configure_qr(None, 0, false).expect("deactivate");
        assert!(!db.qr_status(100).expect("status").0);
    }

    #[test]
    fn hourly_expiry_revokes_generation_and_is_idempotent() {
        let db = crate::db::Db::open_in_memory().expect("db");
        let secret = qr_secret();
        db.configure_qr(Some(&unix_hash(&secret)), 4_000_000_000, true)
            .expect("activate");
        let before = db.qr_status(4_000_000_000).expect("status").3;
        assert!(db
            .expire_qr_generation_if_due(4_000_000_000, QR_ROTATION_INTERVAL)
            .expect("expire"));
        let after = db.qr_status(4_000_000_000).expect("status");
        assert!(!after.0);
        assert_eq!(after.3, before + 1);
        assert!(!db
            .expire_qr_generation_if_due(4_000_000_001, QR_ROTATION_INTERVAL)
            .expect("expire again"));
    }

    #[test]
    fn concurrent_exchange_consumes_single_use() {
        let db = crate::db::Db::open_in_memory().expect("db");
        let secret = qr_secret();
        db.configure_qr(Some(&unix_hash(&secret)), 1_000, true)
            .expect("activate");
        let db = std::sync::Arc::new(db);
        let results = std::thread::scope(|scope| {
            (0..8)
                .map(|i| {
                    let db = std::sync::Arc::clone(&db);
                    let hash = unix_hash(&secret);
                    scope.spawn(move || {
                        db.exchange_qr(
                            &hash,
                            "Ana",
                            "guitar",
                            1,
                            &format!("r{i}"),
                            200,
                            &format!("f{i}"),
                            &format!("j{i}"),
                            200,
                        )
                        .is_ok()
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|h| h.join().expect("thread"))
                .filter(|ok| *ok)
                .count()
        });
        assert_eq!(results, 1);
        assert_eq!(db.qr_status(1).expect("status").2, 0);
    }
}
