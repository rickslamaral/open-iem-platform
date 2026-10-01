//! Built-in preset catalog and bounded channel preset application.

use axum::{
    extract::{Path, State},
    Json,
};
use control_protocol::Role;
use mix_engine::MAX_CHANNELS;
use serde::Deserialize;
use serde::Serialize;

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};

/// Immutable preset catalog entry.
#[derive(Clone, Debug, Serialize)]
pub struct PresetSummary {
    /// Stable preset identifier.
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Preset scope.
    pub kind: &'static str,
    /// Human-readable description.
    pub description: &'static str,
}

/// Canonical built-in presets. Catalog and application use same allowlist.
const BUILTIN_PRESETS: [PresetSummary; 2] = [
    PresetSummary {
        id: "default-vocal",
        name: "Vocal — Default",
        kind: "channel",
        description: "Safe neutral starting point for vocal channels.",
    },
    PresetSummary {
        id: "default-instrument",
        name: "Instrument — Default",
        kind: "channel",
        description: "Safe neutral starting point for instrument channels.",
    },
];

/// Response containing immutable preset catalog entries.
#[derive(Debug, Serialize)]
pub struct PresetsResponse {
    /// Available presets.
    pub presets: Vec<PresetSummary>,
}

/// GET /api/v1/presets — read-only catalog visible to Musician/Engineer/Admin.
///
/// Built-in entries are deliberately immutable.
///
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Musician role. The catalog is immutable;
/// persistence and apply routes require a separate schema and authorization design.
#[allow(clippy::unused_async)]
pub async fn list_presets(
    State(_state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<Json<PresetsResponse>, ApiError> {
    require_min_role(&claims, Role::Musician)?;
    Ok(Json(PresetsResponse {
        presets: BUILTIN_PRESETS.to_vec(),
    }))
}

/// Request selecting target channel for a built-in preset.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyPresetRequest {
    /// Input channel slot.
    pub channel_index: u8,
}

/// Apply built-in channel preset to one input channel.
///
/// Only Engineer/Admin may mutate state. Presets remain a server-side allowlist;
/// arbitrary DSP configuration never crosses this boundary.
///
/// # Errors
/// Returns `ApiError::Forbidden` for Musician, or `ApiError::BadRequest` for
/// unknown preset, invalid channel or failed control dispatch.
#[allow(clippy::unused_async)]
pub async fn apply_preset(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(preset_id): Path<String>,
    Json(body): Json<ApplyPresetRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    if !BUILTIN_PRESETS.iter().any(|preset| preset.id == preset_id) {
        return Err(ApiError::BadRequest("unknown preset".to_owned()));
    }
    if usize::from(body.channel_index) >= MAX_CHANNELS {
        return Err(ApiError::BadRequest("channel is not available".to_owned()));
    }
    let mut ctrl = state
        .control
        .lock()
        .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
    let revision = ctrl
        .apply_channel_preset(body.channel_index, 0.0, false)
        .map_err(|message| ApiError::BadRequest(message.to_owned()))?;
    Ok(Json(serde_json::json!({
        "preset_id": preset_id,
        "channel_index": body.channel_index,
        "revision": revision,
        "applied": true,
    })))
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{generate_refresh_token, hash_password, token_to_storage_key, JwtKeys},
        db::Db,
        middleware::jwt_auth,
        routes::presets::{apply_preset, list_presets},
        security::validate_origin,
        state::AppState,
    };
    use axum::{
        middleware,
        routing::{get, post},
        Router,
    };
    use axum_test::TestServer;
    use control_protocol::Role;
    use control_server::ControlState;
    use serde_json::json;
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
        let private_path = std::env::temp_dir().join(format!("open-iem-presets-test-{nonce}.pem"));
        let public_path =
            std::env::temp_dir().join(format!("open-iem-presets-test-{nonce}.pub.pem"));
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
            .route("/api/v1/presets", get(list_presets))
            .route("/api/v1/presets/{preset_id}/apply", post(apply_preset))
            .layer(middleware::from_fn_with_state(state.clone(), jwt_auth));

        let app = Router::new()
            .merge(protected)
            .with_state(state.clone())
            .layer(middleware::from_fn(validate_origin));

        (TestServer::new(app), state)
    }

    fn seed_user_token(state: &AppState, username: &str, role: Role) -> String {
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
        state
            .jwt
            .issue_with_session(username, user_id, role, &jti, Some(session_id))
            .expect("token issued")
    }

    #[tokio::test]
    async fn list_presets_musician_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus1", Role::Musician);
        let resp = server
            .get("/api/v1/presets")
            .authorization_bearer(token)
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert!(body["presets"].is_array());
        let presets = body["presets"].as_array().expect("array");
        assert!(!presets.is_empty(), "catalog must have at least one preset");
        assert!(presets[0]["id"].is_string());
    }

    #[tokio::test]
    async fn list_presets_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng1", Role::Engineer);
        let resp = server
            .get("/api/v1/presets")
            .authorization_bearer(token)
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert!(body["presets"].is_array());
    }

    #[tokio::test]
    async fn list_presets_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server.get("/api/v1/presets").await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn apply_preset_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng2", Role::Engineer);
        let resp = server
            .post("/api/v1/presets/default-vocal/apply")
            .authorization_bearer(token)
            .json(&json!({ "channel_index": 0_u8 }))
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["preset_id"], "default-vocal");
        assert_eq!(body["channel_index"], 0);
        assert_eq!(body["applied"], true);
        assert!(body["revision"].is_number());
    }

    #[tokio::test]
    async fn apply_preset_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus2", Role::Musician);
        let resp = server
            .post("/api/v1/presets/default-vocal/apply")
            .authorization_bearer(token)
            .json(&json!({ "channel_index": 0_u8 }))
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn apply_preset_unknown_preset_rejected() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng3", Role::Engineer);
        let resp = server
            .post("/api/v1/presets/nonexistent-preset/apply")
            .authorization_bearer(token)
            .json(&json!({ "channel_index": 0_u8 }))
            .await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn apply_preset_out_of_range_channel_rejected() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng4", Role::Engineer);
        // MAX_CHANNELS = 8; channel index 8 is out of range
        let resp = server
            .post("/api/v1/presets/default-vocal/apply")
            .authorization_bearer(token)
            .json(&json!({ "channel_index": 8_u8 }))
            .await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn apply_preset_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server
            .post("/api/v1/presets/default-vocal/apply")
            .json(&json!({ "channel_index": 0_u8 }))
            .await;
        resp.assert_status_unauthorized();
    }
}
