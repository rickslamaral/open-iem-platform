//! Channel REST handlers: `GET /api/v1/state`, `PUT /api/v1/channels/:index`.

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

fn build_snapshot(
    state: &control_server::ControlState,
    visible_mix: Option<usize>,
) -> StateResponse {
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
        ServerMessage::SendAck { .. } => {
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
        ServerMessage::SendAck { .. } => {
            Err(ApiError::Internal("unexpected server message".to_owned()))
        }
    }
}
