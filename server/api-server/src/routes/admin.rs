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

#[cfg(test)]
mod tests {
    use crate::{
        auth::{generate_refresh_token, hash_password, token_to_storage_key, JwtKeys},
        db::Db,
        middleware::jwt_auth,
        routes::admin::{delete_user, list_sessions, list_users, revoke_session},
        security::validate_origin,
        state::AppState,
    };
    use axum::{
        middleware,
        routing::{delete, get},
        Router,
    };
    use axum_test::TestServer;
    use control_protocol::Role;
    use control_server::ControlState;
    use std::{
        fs,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn test_keys() -> (Vec<u8>, Vec<u8>) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be valid")
            .as_nanos();
        let private_path = std::env::temp_dir().join(format!("open-iem-admin-test-{nonce}.pem"));
        let public_path = std::env::temp_dir().join(format!("open-iem-admin-test-{nonce}.pub.pem"));
        let _ = fs::remove_file(&private_path);
        let _ = fs::remove_file(&public_path);
        Command::new("openssl")
            .args(["genpkey", "-algorithm", "ed25519", "-out"])
            .arg(&private_path)
            .status()
            .expect("openssl must be installed");
        Command::new("openssl")
            .args(["pkey", "-in"])
            .arg(&private_path)
            .args(["-pubout", "-out"])
            .arg(&public_path)
            .status()
            .expect("openssl public-key export");
        let private = fs::read(&private_path).expect("private key must be readable");
        let public = fs::read(&public_path).expect("public key must be readable");
        let _ = fs::remove_file(private_path);
        let _ = fs::remove_file(public_path);
        (private, public)
    }

    fn build_test_app() -> (TestServer, AppState) {
        let (private_pem, public_pem) = test_keys();
        let jwt = JwtKeys::from_ed_pem(&private_pem, &public_pem).expect("test PEM valid");
        let db = Db::open_in_memory().expect("in-memory DB");
        let control = ControlState::new();
        let state = AppState::new(control, db, jwt);

        let protected = Router::new()
            .route("/api/v1/admin/users", get(list_users))
            .route("/api/v1/admin/users/{id}", delete(delete_user))
            .route("/api/v1/admin/sessions", get(list_sessions))
            .route("/api/v1/admin/sessions/{id}", delete(revoke_session))
            .layer(middleware::from_fn_with_state(state.clone(), jwt_auth));

        let app = Router::new()
            .merge(protected)
            .with_state(state.clone())
            .layer(middleware::from_fn(validate_origin));

        (TestServer::new(app), state)
    }

    fn seed_user_token(state: &AppState, username: &str, role: Role) -> (i64, String) {
        let pw_hash = hash_password("pass").expect("hash");
        state
            .db
            .create_user(username, &pw_hash, role)
            .expect("create user");
        let (user_id, _, _, _) = state.db.find_user(username).expect("user exists");
        let jti = uuid::Uuid::new_v4().to_string();
        let raw_refresh = generate_refresh_token();
        let session_id = state
            .db
            .create_session_with_access(
                user_id,
                &token_to_storage_key(&raw_refresh),
                4_000_000_000,
                &uuid::Uuid::new_v4().to_string(),
                &jti,
                4_000_000_000,
            )
            .expect("session created");
        let token = state
            .jwt
            .issue_with_session(username, user_id, role, &jti, Some(session_id))
            .expect("token issued");
        (user_id, token)
    }

    #[tokio::test]
    async fn list_users_admin_ok() {
        let (server, state) = build_test_app();
        let (_id, token) = seed_user_token(&state, "admin1", Role::Admin);
        let resp = server
            .get("/api/v1/admin/users")
            .authorization_bearer(token)
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert!(body.is_array());
    }

    #[tokio::test]
    async fn list_users_engineer_forbidden() {
        let (server, state) = build_test_app();
        let (_id, token) = seed_user_token(&state, "eng1", Role::Engineer);
        let resp = server
            .get("/api/v1/admin/users")
            .authorization_bearer(token)
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn list_users_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server.get("/api/v1/admin/users").await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn delete_user_admin_ok() {
        let (server, state) = build_test_app();
        let (_admin_id, admin_token) = seed_user_token(&state, "admin2", Role::Admin);
        let (target_id, _) = seed_user_token(&state, "target_user", Role::Musician);
        let resp = server
            .delete(&format!("/api/v1/admin/users/{target_id}"))
            .authorization_bearer(admin_token)
            .await;
        resp.assert_status(axum::http::StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn delete_user_cannot_delete_self() {
        let (server, state) = build_test_app();
        let (admin_id, admin_token) = seed_user_token(&state, "admin3", Role::Admin);
        let resp = server
            .delete(&format!("/api/v1/admin/users/{admin_id}"))
            .authorization_bearer(admin_token)
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn delete_user_engineer_forbidden() {
        let (server, state) = build_test_app();
        let (_eng_id, eng_token) = seed_user_token(&state, "eng2", Role::Engineer);
        let (_target_id, _) = seed_user_token(&state, "target2", Role::Musician);
        let resp = server
            .delete("/api/v1/admin/users/999")
            .authorization_bearer(eng_token)
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn list_sessions_admin_ok() {
        let (server, state) = build_test_app();
        let (_id, token) = seed_user_token(&state, "admin4", Role::Admin);
        let resp = server
            .get("/api/v1/admin/sessions")
            .authorization_bearer(token)
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert!(body.is_array());
    }

    #[tokio::test]
    async fn list_sessions_engineer_forbidden() {
        let (server, state) = build_test_app();
        let (_id, token) = seed_user_token(&state, "eng3", Role::Engineer);
        let resp = server
            .get("/api/v1/admin/sessions")
            .authorization_bearer(token)
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn revoke_session_admin_ok() {
        let (server, state) = build_test_app();
        let (admin_id, admin_token) = seed_user_token(&state, "admin5", Role::Admin);
        // Create a second session for admin5 to revoke (not the current one, which has no stored id in token)
        let pw_hash = hash_password("pass").expect("hash");
        let raw_refresh = generate_refresh_token();
        let session_id = state
            .db
            .create_session_with_access(
                admin_id,
                &token_to_storage_key(&raw_refresh),
                4_000_000_000,
                &uuid::Uuid::new_v4().to_string(),
                &uuid::Uuid::new_v4().to_string(),
                4_000_000_000,
            )
            .expect("extra session created");
        let _ = pw_hash;
        let resp = server
            .delete(&format!("/api/v1/admin/sessions/{session_id}"))
            .authorization_bearer(admin_token)
            .await;
        resp.assert_status(axum::http::StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn revoke_session_engineer_forbidden() {
        let (server, state) = build_test_app();
        let (_id, token) = seed_user_token(&state, "eng4", Role::Engineer);
        let resp = server
            .delete("/api/v1/admin/sessions/1")
            .authorization_bearer(token)
            .await;
        resp.assert_status_forbidden();
    }
}
