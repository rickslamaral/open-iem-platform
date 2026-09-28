//! Channel REST handlers: `GET /api/v1/state`, `GET /api/v1/channels`, `PUT /api/v1/channels/:index`.

#![allow(clippy::unused_async)]

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use control_protocol::{ClientMessage, Envelope, Role, ServerMessage};
use mix_engine::{GAIN_DB_MAX, GAIN_DB_MIN};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Full control-plane state response.
#[derive(Serialize)]
pub struct StateResponse {
    /// Contract schema version.
    pub schema_version: u8,
    /// Current state revision.
    pub revision: u64,
    /// Configured input channels.
    pub channels: Vec<ChannelSnapshot>,
    /// Configured mixes visible to caller.
    pub mixes: Vec<MixSnapshot>,
}

/// Serializable channel state.
#[derive(Serialize)]
pub struct ChannelSnapshot {
    /// Stable slot index.
    pub index: usize,
    /// Channel ID.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Input gain in dB.
    pub gain_db: f32,
    /// Global mute.
    pub muted: bool,
    /// Lock flag.
    pub locked: bool,
    /// Enabled flag.
    pub enabled: bool,
    /// Channel revision.
    pub revision: u64,
}

/// Serializable mix state.
#[derive(Serialize)]
pub struct MixSnapshot {
    /// Stable slot index.
    pub index: usize,
    /// Mix ID.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Master gain in dB.
    pub master_gain_db: f32,
    /// Master mute.
    pub master_muted: bool,
    /// Mix revision.
    pub revision: u64,
    /// Configured sends.
    pub sends: Vec<SendSnapshot>,
}

/// Serializable mix send state.
#[derive(Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct SendSnapshot {
    /// Channel slot index.
    pub channel_index: usize,
    /// Channel ID.
    pub channel_id: u32,
    /// Mix ID.
    pub mix_id: u32,
    /// Send gain in dB.
    pub gain_db: f32,
    /// Pan position.
    pub pan: f32,
    /// Mute flag.
    pub muted: bool,
    /// Solo flag.
    pub solo: bool,
    /// Enabled flag.
    pub enabled: bool,
    /// Lock flag.
    pub locked: bool,
    /// Send revision.
    pub revision: u64,
}

const STATE_SCHEMA_VERSION: u8 = 1;

fn build_channels(state: &control_server::ControlState) -> Vec<ChannelSnapshot> {
    let mut channels = Vec::new();
    state.for_each_channel(|index, channel| {
        channels.push(ChannelSnapshot {
            index,
            id: channel.id,
            name: channel.name().to_owned(),
            gain_db: channel.gain_db(),
            muted: channel.muted,
            locked: channel.locked,
            enabled: channel.enabled,
            revision: channel.revision(),
        });
    });
    channels
}

fn build_snapshot(
    state: &control_server::ControlState,
    visible_mix: Option<usize>,
) -> StateResponse {
    let channels = build_channels(state);
    let mut mixes = Vec::new();
    state.for_each_mix(|index, mix| {
        if visible_mix.is_some_and(|allowed| allowed != index) {
            return;
        }
        let mut sends = Vec::new();
        for channel_index in 0..mix_engine::MAX_CHANNELS {
            if let Some(send) = mix.send(channel_index) {
                sends.push(SendSnapshot {
                    channel_index,
                    channel_id: send.channel_id,
                    mix_id: send.mix_id,
                    gain_db: send.gain_db(),
                    pan: send.pan(),
                    muted: send.muted,
                    solo: send.solo,
                    enabled: send.enabled,
                    locked: send.locked,
                    revision: send.revision(),
                });
            }
        }
        mixes.push(MixSnapshot {
            index,
            id: mix.id,
            name: mix.name().to_owned(),
            master_gain_db: mix.master_gain_db(),
            master_muted: mix.master_muted,
            revision: mix.revision(),
            sends,
        });
    });
    StateResponse {
        schema_version: STATE_SCHEMA_VERSION,
        revision: state.revision(),
        channels,
        mixes,
    }
}

/// `GET /api/v1/state` — returns current engine revision.
///
/// Requires at least Musician role.
///
/// # Errors
/// Returns `ApiError::Forbidden` if role insufficient.
pub async fn get_state(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Musician)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    let visible_mix = if claims.role == Role::Musician {
        Some(
            state
                .db
                .get_user_assigned_mix(claims.user_id)?
                .unwrap_or(mix_engine::MAX_MIXES),
        )
    } else {
        None
    };
    let ctrl = state
        .control
        .lock()
        .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
    Ok(Json(build_snapshot(&ctrl, visible_mix)))
}

/// Response containing configured input channels.
#[derive(Serialize)]
pub struct ChannelsResponse {
    /// Contract schema version.
    pub schema_version: u8,
    /// Current state revision.
    pub revision: u64,
    /// Configured input channels.
    pub channels: Vec<ChannelSnapshot>,
}

/// `GET /api/v1/channels` — returns configured input channels.
///
/// Requires at least Musician role.
///
/// # Errors
/// Returns `ApiError::Forbidden` if role insufficient.
pub async fn list_channels(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Musician)?;
    let ctrl = state
        .control
        .lock()
        .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
    Ok(Json(ChannelsResponse {
        schema_version: STATE_SCHEMA_VERSION,
        revision: ctrl.revision(),
        channels: build_channels(&ctrl),
    }))
}

/// Build a state snapshot from one consistent control-state lock.
#[must_use]
pub fn snapshot_for_test(state: &control_server::ControlState) -> StateResponse {
    build_snapshot(state, None)
}

/// Channel gain update request.
#[derive(Deserialize)]
pub struct SetGainRequest {
    /// Gain in dBFS.
    pub gain_db: f32,
}

/// Channel mute update request.
#[derive(Deserialize)]
pub struct SetMuteRequest {
    /// Mute flag.
    pub muted: bool,
}

/// Channel state response.
#[derive(Serialize)]
pub struct ChannelResponse {
    /// Channel index.
    pub index: usize,
    /// Gain in dBFS.
    pub gain_db: f32,
    /// Mute state.
    pub muted: bool,
    /// Engine state revision after update.
    pub revision: u64,
}

/// `PUT /api/v1/channels/:index/gain` — set channel gain.
///
/// Requires at least Engineer role.
///
/// # Errors
/// Returns `ApiError::Forbidden` if role insufficient, or `ApiError::BadRequest` on invalid gain.
pub async fn set_channel_gain(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(index): Path<u8>,
    Json(body): Json<SetGainRequest>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    if !body.gain_db.is_finite() {
        return Err(ApiError::BadRequest("gain_db must be finite".to_owned()));
    }
    if !(GAIN_DB_MIN..=GAIN_DB_MAX).contains(&body.gain_db) {
        return Err(ApiError::BadRequest(format!(
            "gain_db must be between {GAIN_DB_MIN} and {GAIN_DB_MAX} dB"
        )));
    }
    let request_id = Uuid::new_v4().to_string();
    let envelope = Envelope::new(
        request_id,
        ClientMessage::SetChannelGain {
            channel: index,
            gain_db: body.gain_db,
        },
    );
    let response = {
        let mut ctrl = state
            .control
            .lock()
            .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
        ctrl.dispatch(envelope)
    };
    match response.payload {
        ServerMessage::State { revision } => Ok(Json(serde_json::json!({ "revision": revision }))),
        ServerMessage::Error { code, message } => {
            Err(ApiError::BadRequest(format!("{code}: {message}")))
        }
        ServerMessage::SendAck { .. }
        | ServerMessage::MasterAck { .. }
        | ServerMessage::EqBandAck { .. } => {
            Err(ApiError::Internal("unexpected server message".to_owned()))
        }
    }
}

/// `PUT /api/v1/channels/:index/mute` — set channel mute.
///
/// Requires at least Engineer role.
///
/// # Errors
/// Returns `ApiError::Forbidden` if role insufficient.
pub async fn set_channel_mute(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(index): Path<u8>,
    Json(body): Json<SetMuteRequest>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let request_id = Uuid::new_v4().to_string();
    let envelope = Envelope::new(
        request_id,
        ClientMessage::SetChannelMute {
            channel: index,
            muted: body.muted,
        },
    );
    let response = {
        let mut ctrl = state
            .control
            .lock()
            .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
        ctrl.dispatch(envelope)
    };
    match response.payload {
        ServerMessage::State { revision } => Ok(Json(serde_json::json!({ "revision": revision }))),
        ServerMessage::Error { code, message } => {
            Err(ApiError::BadRequest(format!("{code}: {message}")))
        }
        ServerMessage::SendAck { .. }
        | ServerMessage::MasterAck { .. }
        | ServerMessage::EqBandAck { .. } => {
            Err(ApiError::Internal("unexpected server message".to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{generate_refresh_token, hash_password, token_to_storage_key, JwtKeys},
        db::Db,
        middleware::jwt_auth,
        routes::channels::{get_state, list_channels, set_channel_gain, set_channel_mute},
        security::validate_origin,
        state::AppState,
    };
    use axum::{
        middleware,
        routing::{get, put},
        Router,
    };
    use axum_test::TestServer;
    use control_protocol::Role;
    use control_server::ControlState;
    use serde_json::json;
    use std::{fs, process::Command};
    use uuid::Uuid;

    fn test_keys() -> (Vec<u8>, Vec<u8>) {
        let nonce = Uuid::new_v4();
        let private_path = std::env::temp_dir().join(format!("open-iem-channels-test-{nonce}.pem"));
        let public_path =
            std::env::temp_dir().join(format!("open-iem-channels-test-{nonce}.pub.pem"));
        let _ = fs::remove_file(&private_path);
        let _ = fs::remove_file(&public_path);
        Command::new("openssl")
            .args(["genpkey", "-algorithm", "ed25519", "-out"])
            .arg(&private_path)
            .status()
            .expect("openssl must be installed")
            .success()
            .then_some(())
            .expect("openssl key generation must succeed");
        Command::new("openssl")
            .args(["pkey", "-in"])
            .arg(&private_path)
            .args(["-pubout", "-out"])
            .arg(&public_path)
            .status()
            .expect("openssl public-key export")
            .success()
            .then_some(())
            .expect("openssl public-key export must succeed");
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
        let state = AppState::new(ControlState::new(), db, jwt);

        let protected = Router::new()
            .route("/api/v1/state", get(get_state))
            .route("/api/v1/channels", get(list_channels))
            .route("/api/v1/channels/{index}/gain", put(set_channel_gain))
            .route("/api/v1/channels/{index}/mute", put(set_channel_mute))
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
    async fn get_state_musician_returns_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "musician1", Role::Musician);
        let resp = server
            .get("/api/v1/state")
            .authorization_bearer(token)
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["schema_version"], 1);
        assert!(body["channels"].is_array());
        assert!(body["mixes"].is_array());
    }

    #[tokio::test]
    async fn get_state_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server.get("/api/v1/state").await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn list_channels_musician_returns_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "musician2", Role::Musician);
        let resp = server
            .get("/api/v1/channels")
            .authorization_bearer(token)
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["schema_version"], 1);
        assert!(body["channels"].is_array());
    }

    #[tokio::test]
    async fn list_channels_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server.get("/api/v1/channels").await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn set_channel_gain_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "engineer1", Role::Engineer);
        let resp = server
            .put("/api/v1/channels/0/gain")
            .authorization_bearer(token)
            .json(&json!({ "gain_db": 0.0_f32 }))
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert!(body["revision"].is_number());
    }

    #[tokio::test]
    async fn set_channel_gain_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "musician3", Role::Musician);
        let resp = server
            .put("/api/v1/channels/0/gain")
            .authorization_bearer(token)
            .json(&json!({ "gain_db": 0.0_f32 }))
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn set_channel_gain_out_of_range_rejected() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "engineer2", Role::Engineer);
        let resp = server
            .put("/api/v1/channels/0/gain")
            .authorization_bearer(token)
            .json(&json!({ "gain_db": 200.0_f32 }))
            .await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn set_channel_gain_non_finite_rejected() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "engineer3", Role::Engineer);
        // JSON string for gain_db is a type error; axum returns 422.
        let raw = r#"{"gain_db": "inf"}"#;
        let resp = server
            .put("/api/v1/channels/0/gain")
            .authorization_bearer(token)
            .content_type("application/json")
            .bytes(raw.as_bytes().into())
            .await;
        assert!(
            resp.status_code().is_client_error(),
            "non-finite gain must be a client error, got {}",
            resp.status_code()
        );
    }

    #[tokio::test]
    async fn set_channel_mute_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "engineer4", Role::Engineer);
        let resp = server
            .put("/api/v1/channels/0/mute")
            .authorization_bearer(token)
            .json(&json!({ "muted": true }))
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert!(body["revision"].is_number());
    }

    #[tokio::test]
    async fn set_channel_mute_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "musician4", Role::Musician);
        let resp = server
            .put("/api/v1/channels/0/mute")
            .authorization_bearer(token)
            .json(&json!({ "muted": false }))
            .await;
        resp.assert_status_forbidden();
    }
}
