//! Admin-only route handlers for user and session management.

#![allow(clippy::unused_async)]

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use control_protocol::Role;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// JSON body for a single user entry returned by the list-users endpoint.
#[derive(Serialize)]
pub struct UserEntry {
    /// Numeric user ID.
    pub id: i64,
    /// Login username.
    pub username: String,
    /// Role string (e.g. `ADMIN`, `ENGINEER`, `MUSICIAN`).
    pub role: String,
}

/// JSON body for a single session entry returned by the list-sessions endpoint.
#[derive(Serialize)]
pub struct SessionEntry {
    /// Numeric session (refresh-token) ID.
    pub id: i64,
    /// ID of the owning user.
    pub user_id: i64,
    /// Unix timestamp (seconds) when this session expires.
    pub expires_at: u64,
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// List all registered users.
///
/// # Errors
/// Returns `ApiError::Forbidden` if caller is not Admin.
/// Returns `ApiError::Internal` on DB error.
pub async fn list_users(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Admin)?;
    let users = state.db.list_users()?;
    let entries: Vec<UserEntry> = users
        .into_iter()
        .map(|(id, username, role)| UserEntry {
            id,
            username,
            role: format!("{role:?}").to_uppercase(),
        })
        .collect();
    Ok(Json(entries))
}

/// Delete a user by numeric ID.
///
/// # Errors
/// Returns `ApiError::Forbidden` if caller is not Admin.
/// Returns `ApiError::Forbidden` if caller attempts to delete their own account.
/// Returns `ApiError::NotFound` if user does not exist.
/// Returns `ApiError::Internal` on DB error.
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Path(user_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Admin)?;
    if user_id == claims.user_id {
        return Err(ApiError::Forbidden("Cannot delete your own account"));
    }
    state.db.delete_user_with_sessions(user_id)?;
    Ok(StatusCode::NO_CONTENT)
}

/// List all active (non-revoked, non-expired) refresh-token sessions.
///
/// # Errors
/// Returns `ApiError::Forbidden` if caller is not Admin.
/// Returns `ApiError::Internal` on DB error.
pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Admin)?;
    let sessions = state.db.list_active_sessions(unix_now())?;
    let entries: Vec<SessionEntry> = sessions
        .into_iter()
        .map(|(id, user_id, expires_at)| SessionEntry {
            id,
            user_id,
            expires_at,
        })
        .collect();
    Ok(Json(entries))
}

/// Revoke a refresh-token session by its numeric ID.
///
/// # Errors
/// Returns `ApiError::Forbidden` if caller is not Admin.
/// Returns `ApiError::NotFound` if session does not exist.
/// Returns `ApiError::Internal` on DB error.
pub async fn revoke_session(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Path(session_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Admin)?;
    state.db.revoke_session_by_id(session_id)?;
    Ok(StatusCode::NO_CONTENT)
}
