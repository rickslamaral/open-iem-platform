//! Auth routes: `POST /api/v1/auth/login`, `POST /api/v1/auth/refresh`, `POST /api/v1/auth/logout`.

#![allow(clippy::unused_async)]

use crate::{
    auth::{
        generate_refresh_token, hash_password, token_to_storage_key, verify_password, JwtClaims,
        REFRESH_TOKEN_TTL_S,
    },
    error::ApiError,
    middleware::require_min_role,
    state::AppState,
};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use control_protocol::Role;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const MAX_USERNAME_BYTES: usize = 128;
const MAX_PASSWORD_BYTES: usize = 1024;

fn validate_credentials(username: &str, password: &str) -> Result<(), ApiError> {
    if username.is_empty() || username.len() > MAX_USERNAME_BYTES {
        return Err(ApiError::BadRequest(
            "username length is invalid".to_owned(),
        ));
    }
    if password.is_empty() || password.len() > MAX_PASSWORD_BYTES {
        return Err(ApiError::BadRequest(
            "password length is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Parse a named cookie from the `Cookie` request header.
/// Returns the value as `&str` if found.
fn extract_cookie_value<'a>(headers: &'a axum::http::HeaderMap, name: &str) -> Option<&'a str> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some((k, v)) = pair.split_once('=') {
            if k.trim() == name {
                return Some(v.trim());
            }
        }
    }
    None
}

/// Login request body.
#[derive(Deserialize)]
pub struct LoginRequest {
    /// Username.
    pub username: String,
    /// Plaintext password (transmitted over HTTPS only).
    pub password: String,
}

/// Login response body.
///
/// The refresh token is NOT returned here — it is delivered exclusively via
/// `Set-Cookie: refresh_token=...; HttpOnly; Secure; SameSite=Strict` to
/// prevent JavaScript access. The access token is short-lived (15 min) and
/// is safe to return in the body.
#[derive(Serialize)]
pub struct LoginResponse {
    /// Short-lived access token (JWT, 15 min). Store in memory only — not localStorage.
    pub access_token: String,
    /// Role granted.
    pub role: Role,
}

/// `POST /api/v1/auth/login`
///
/// Validates username + password, issues access + refresh tokens.
/// Refresh token is delivered via `HttpOnly` cookie only, not in the body.
///
/// # Errors
/// Returns `ApiError::Unauthorized` on bad credentials.
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_credentials(&body.username, &body.password)?;
    let (user_id, pw_hash, role) = state.db.find_user(&body.username)?;
    verify_password(&body.password, &pw_hash)?;

    let jti = Uuid::new_v4().to_string();
    let raw_refresh = generate_refresh_token();
    let family = Uuid::new_v4().to_string();
    let now = unix_now();
    let expires_at = now + REFRESH_TOKEN_TTL_S;
    let token_hash = token_to_storage_key(&raw_refresh);
    let session_id = state.db.create_session_with_access(
        user_id,
        &token_hash,
        expires_at,
        &family,
        &jti,
        now + crate::auth::ACCESS_TOKEN_TTL_S,
    )?;
    let access =
        match state
            .jwt
            .issue_with_session(&body.username, user_id, role, &jti, Some(session_id))
        {
            Ok(access) => access,
            Err(error) => {
                state
                    .db
                    .cleanup_session_after_signing_failure(&token_hash, &jti)?;
                return Err(error);
            }
        };

    Ok((
        StatusCode::OK,
        [(
            header::SET_COOKIE,
            format!(
                "refresh_token={raw_refresh}; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth/refresh; Max-Age={REFRESH_TOKEN_TTL_S}"
            ),
        )],
        Json(LoginResponse {
            access_token: access,
            role,
        }),
    ))
}

/// Refresh response body.
///
/// New refresh token is delivered via `HttpOnly` cookie only.
#[derive(Serialize)]
pub struct RefreshResponse {
    /// New access token.
    pub access_token: String,
}

/// `POST /api/v1/auth/refresh`
///
/// Reads refresh token from `Cookie: refresh_token=...` header.
/// Rotates refresh token and issues a new access token.
/// New refresh token is delivered via `HttpOnly` cookie only.
///
/// # Errors
/// Returns `ApiError::Unauthorized` on invalid, expired, or replayed refresh token.
pub async fn refresh(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let raw_refresh = extract_cookie_value(&headers, "refresh_token")
        .ok_or(ApiError::Unauthorized("missing refresh_token cookie"))?;
    let _refresh_guard = state
        .refresh_lock
        .lock()
        .map_err(|_| ApiError::Internal("refresh lock poisoned".to_owned()))?;

    let now = unix_now();
    let token_hash = token_to_storage_key(raw_refresh);
    let jti = Uuid::new_v4().to_string();
    let raw_new_refresh = generate_refresh_token();
    let expires_at = now + REFRESH_TOKEN_TTL_S;
    let new_token_hash = token_to_storage_key(&raw_new_refresh);
    let (user_id, _, new_session_id) = state.db.rotate_refresh_token_with_access_id(
        &token_hash,
        &new_token_hash,
        expires_at,
        now,
        Some(&jti),
        Some(now + crate::auth::ACCESS_TOKEN_TTL_S),
    )?;
    let (username, role) = state.db.find_user_by_id(user_id)?;
    let access =
        match state
            .jwt
            .issue_with_session(&username, user_id, role, &jti, Some(new_session_id))
        {
            Ok(access) => access,
            Err(error) => {
                state.db.restore_refresh_after_signing_failure(
                    &token_hash,
                    &new_token_hash,
                    &jti,
                )?;
                return Err(error);
            }
        };

    Ok((
        StatusCode::OK,
        [(
            header::SET_COOKIE,
            format!(
                "refresh_token={raw_new_refresh}; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth/refresh; Max-Age={REFRESH_TOKEN_TTL_S}"
            ),
        )],
        Json(RefreshResponse {
            access_token: access,
        }),
    ))
}

/// `POST /api/v1/auth/logout`
///
/// Revokes all refresh tokens for the authenticated user.
/// Requires valid access token (via JWT middleware).
///
/// # Errors
/// Returns `ApiError::Internal` on DB failure.
pub async fn logout(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    let (user_id, _, _) = state.db.find_user(&claims.sub)?;
    state.db.revoke_all_for_user(user_id)?;
    // Clear the cookie
    Ok((
        StatusCode::NO_CONTENT,
        [(
            header::SET_COOKIE,
            "refresh_token=; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth/refresh; Max-Age=0".to_owned(),
        )],
    ))
}

/// Admin-only: seed a new user account.
///
/// # Errors
/// Returns `ApiError::Forbidden` if caller is not Admin, or `ApiError::BadRequest` on duplicate.
pub async fn create_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Json(body): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Admin)?;
    validate_credentials(&body.username, &body.password)?;
    let pw_hash = hash_password(&body.password)?;
    state.db.create_user(&body.username, &pw_hash, body.role)?;
    Ok(StatusCode::CREATED)
}

/// Create user request body.
#[derive(Deserialize)]
pub struct CreateUserRequest {
    /// Username.
    pub username: String,
    /// Plaintext password.
    pub password: String,
    /// Role to assign.
    pub role: Role,
}
