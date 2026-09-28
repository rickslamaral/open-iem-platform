//! Operational configuration backup and restore endpoints.

use axum::{extract::State, response::IntoResponse, Json};
use config_backup::{backup, restore, ConfigSnapshot};
use control_protocol::Role;

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};

/// `GET /api/v1/config/backup` — exports current control-plane configuration.
///
/// Requires Engineer or Admin. Credentials and transient DSP state are not
/// included by [`config_backup::backup`].
///
/// # Errors
/// Returns `ApiError::Forbidden` for non-Engineer/Admin callers or when the
/// control-state lock is poisoned.
#[allow(clippy::unused_async)]
pub async fn backup_config(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let control = state
        .control
        .lock()
        .map_err(|_| ApiError::Internal("control state lock poisoned".to_owned()))?;
    Ok(Json(backup(&control)))
}

/// `PUT /api/v1/config/backup` — restores a validated control-plane snapshot.
///
/// Restore is atomic with respect to other control-state mutations. Existing
/// slots absent from snapshot remain unchanged; this preserves documented
/// `config_backup::restore` semantics.
///
/// # Errors
/// Returns `ApiError::Forbidden` for non-Engineer/Admin callers,
/// `ApiError::BadRequest` for an unsupported or invalid snapshot, or
/// `ApiError::Internal` when the control-state lock is poisoned.
#[allow(clippy::unused_async)]
pub async fn restore_config(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Json(snapshot): Json<ConfigSnapshot>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let mut control = state
        .control
        .lock()
        .map_err(|_| ApiError::Internal("control state lock poisoned".to_owned()))?;
    restore(&snapshot, &mut control).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    Ok(Json(
        serde_json::json!({ "restored": true, "version": snapshot.version }),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{generate_refresh_token, hash_password, token_to_storage_key, JwtKeys},
        db::Db,
        middleware::jwt_auth,
        routes::config::{backup_config, restore_config},
        security::validate_origin,
        state::AppState,
    };
    use axum::{
        middleware,
        routing::get,
        Router,
    };
    use axum_test::TestServer;
    use config_backup::ConfigSnapshot;
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
        let private_path = std::env::temp_dir().join(format!("open-iem-config-test-{nonce}.pem"));
        let public_path =
            std::env::temp_dir().join(format!("open-iem-config-test-{nonce}.pub.pem"));
        let _ = fs::remove_file(&private_path);
        let _ = fs::remove_file(&public_path);
        let status = Command::new("openssl")
            .args(["genpkey", "-algorithm", "ed25519", "-out"])
            .arg(&private_path)
            .status()
            .expect("openssl must be installed for integration tests");
        assert!(status.success(), "openssl key generation failed");
        let status = Command::new("openssl")
            .args(["pkey", "-in"])
            .arg(&private_path)
            .args(["-pubout", "-out"])
            .arg(&public_path)
            .status()
            .expect("openssl public-key export failed");
        assert!(status.success(), "openssl public-key export failed");
        let private = fs::read(&private_path).expect("generated private key must be readable");
        let public = fs::read(&public_path).expect("generated public key must be readable");
        let _ = fs::remove_file(private_path);
        let _ = fs::remove_file(public_path);
        (private, public)
    }

    fn build_test_app() -> (TestServer, AppState) {
        let (private_pem, public_pem) = test_keys();
        let jwt = JwtKeys::from_ed_pem(&private_pem, &public_pem).expect("test PEM must be valid");
        let db = Db::open_in_memory().expect("in-memory DB must open");
        let state = AppState::new(ControlState::new(), db, jwt);

        let protected = Router::new()
            .route(
                "/api/v1/config/backup",
                get(backup_config).put(restore_config),
            )
            .layer(middleware::from_fn_with_state(state.clone(), jwt_auth));

        let app = Router::new()
            .merge(protected)
            .with_state(state.clone())
            .layer(middleware::from_fn(validate_origin));

        let server = TestServer::new(app);
        (server, state)
    }

    fn seed_user_token(state: &AppState, username: &str, role: Role) -> String {
        let pw_hash = hash_password("pw").expect("hash must succeed");
        state
            .db
            .create_user(username, &pw_hash, role)
            .expect("create user must succeed");
        let (user_id, _, _, _) = state.db.find_user(username).expect("user must exist");
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
            .expect("session must be created");
        state
            .jwt
            .issue_with_session(username, user_id, role, &jti, Some(session_id))
            .expect("token must be issued")
    }

    fn empty_snapshot() -> ConfigSnapshot {
        ConfigSnapshot {
            version: 1,
            created_at_utc_secs: 0,
            channels: vec![],
            mixes: vec![],
        }
    }

    // --- backup_config (GET /api/v1/config/backup) ---

    #[tokio::test]
    async fn backup_config_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng_backup_ok", Role::Engineer);
        let resp = server
            .get("/api/v1/config/backup")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["version"], 1);
    }

    #[tokio::test]
    async fn backup_config_admin_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "admin_backup_ok", Role::Admin);
        let resp = server
            .get("/api/v1/config/backup")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["version"], 1);
    }

    #[tokio::test]
    async fn backup_config_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus_backup_forbidden", Role::Musician);
        let resp = server
            .get("/api/v1/config/backup")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn backup_config_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server
            .get("/api/v1/config/backup")
            .add_header("Origin", "http://localhost")
            .await;
        resp.assert_status_unauthorized();
    }

    // --- restore_config (PUT /api/v1/config/backup) ---

    #[tokio::test]
    async fn restore_config_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng_restore_ok", Role::Engineer);
        let resp = server
            .put("/api/v1/config/backup")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .json(&empty_snapshot())
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["restored"], true);
        assert_eq!(body["version"], 1);
    }

    #[tokio::test]
    async fn restore_config_admin_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "admin_restore_ok", Role::Admin);
        let resp = server
            .put("/api/v1/config/backup")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .json(&empty_snapshot())
            .await;
        resp.assert_status_success();
    }

    #[tokio::test]
    async fn restore_config_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus_restore_forbidden", Role::Musician);
        let resp = server
            .put("/api/v1/config/backup")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .json(&empty_snapshot())
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn restore_config_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server
            .put("/api/v1/config/backup")
            .add_header("Origin", "http://localhost")
            .json(&empty_snapshot())
            .await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn restore_config_invalid_version_rejected() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng_restore_bad_ver", Role::Engineer);
        let bad_snapshot = ConfigSnapshot {
            version: 99,
            created_at_utc_secs: 0,
            channels: vec![],
            mixes: vec![],
        };
        let resp = server
            .put("/api/v1/config/backup")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .json(&bad_snapshot)
            .await;
        resp.assert_status_bad_request();
    }
}
