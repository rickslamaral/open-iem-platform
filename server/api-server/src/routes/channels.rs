//! Channel REST handlers: `GET /api/v1/state`, `PUT /api/v1/channels/:index`.

#![allow(clippy::unused_async)]

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use control_protocol::{ClientMessage, Envelope, Role, ServerMessage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Full engine state response.
#[derive(Serialize)]
pub struct StateResponse {
    /// Current state revision.
    pub revision: u64,
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
    let ctrl = state
        .control
        .lock()
        .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
    Ok(Json(StateResponse {
        revision: ctrl.revision(),
    }))
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
    }
}
