//! Mix assignment and musician-owned send routes.

#![allow(clippy::missing_errors_doc, clippy::unused_async)]

use crate::{
    auth::JwtClaims, db::Db, error::ApiError, middleware::require_min_role, state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use control_protocol::Role;
use mix_engine::{GAIN_DB_MAX, GAIN_DB_MIN, MAX_CHANNELS, MAX_MIXES};
use serde::{Deserialize, Serialize};

/// Mix assignment response.
#[derive(Serialize)]
pub struct MixAssignment {
    /// Mix slot.
    pub mix_index: usize,
    /// Assigned user ID.
    pub user_id: i64,
    /// Assigned username.
    pub username: String,
}
/// Assignment request.
#[derive(Deserialize)]
pub struct AssignRequest {
    /// User ID.
    pub user_id: i64,
}
/// Gain update request.
#[derive(Deserialize)]
pub struct GainRequest {
    /// Gain in dBFS.
    pub gain_db: f32,
}
/// Pan update request.
#[derive(Deserialize)]
pub struct PanRequest {
    /// Pan from -1 to 1.
    pub pan: f32,
}
/// Mute update request.
#[derive(Deserialize)]
pub struct MuteRequest {
    /// Mute state.
    pub muted: bool,
}
/// Send state response.
#[derive(Serialize)]
pub struct SendState {
    /// Mix slot.
    pub mix_index: usize,
    /// Channel slot.
    pub channel_index: usize,
    /// Gain in dBFS.
    pub gain_db: f32,
    /// Pan.
    pub pan: f32,
    /// Mute state.
    pub muted: bool,
    /// Send revision.
    pub revision: u64,
}

fn check_mix_ownership(claims: &JwtClaims, mix_index: usize, db: &Db) -> Result<(), ApiError> {
    if matches!(claims.role, Role::Admin | Role::Engineer) {
        return Ok(());
    }
    if db.get_user_assigned_mix(claims.user_id)? == Some(mix_index) {
        Ok(())
    } else {
        Err(ApiError::Forbidden("musician does not own this mix"))
    }
}
fn validate_indexes(mix_index: usize, channel_index: usize) -> Result<(), ApiError> {
    if mix_index >= MAX_MIXES || channel_index >= MAX_CHANNELS {
        Err(ApiError::BadRequest("index out of range".to_owned()))
    } else {
        Ok(())
    }
}

/// List assignments.
pub async fn list_mixes(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<Json<Vec<MixAssignment>>, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    Ok(Json(
        state
            .db
            .list_mix_assignments()?
            .into_iter()
            .map(|(mix_index, user_id, username)| MixAssignment {
                mix_index,
                user_id,
                username,
            })
            .collect(),
    ))
}
/// Assign user to mix.
pub async fn assign_mix(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(index): Path<usize>,
    Json(body): Json<AssignRequest>,
) -> Result<Json<MixAssignment>, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    if index >= MAX_MIXES || body.user_id <= 0 {
        return Err(ApiError::BadRequest("invalid mix assignment".to_owned()));
    }
    state.db.assign_mix(index, body.user_id)?;
    let username = state.db.find_user_by_id(body.user_id)?.0;
    Ok(Json(MixAssignment {
        mix_index: index,
        user_id: body.user_id,
        username,
    }))
}
/// Unassign mix.
pub async fn unassign_mix(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(index): Path<usize>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    state.db.unassign_mix(index)?;
    Ok(StatusCode::NO_CONTENT)
}
fn read_state(
    state: &AppState,
    mix_index: usize,
    channel_index: usize,
) -> Result<SendState, ApiError> {
    let ctrl = state
        .control
        .lock()
        .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
    let send = ctrl
        .mix(mix_index)
        .and_then(|mix| mix.get_send(channel_index));
    Ok(match send {
        Some(s) => SendState {
            mix_index,
            channel_index,
            gain_db: s.gain_db(),
            pan: s.pan(),
            muted: s.muted,
            revision: s.revision(),
        },
        None => SendState {
            mix_index,
            channel_index,
            gain_db: 0.0,
            pan: 0.0,
            muted: false,
            revision: 0,
        },
    })
}
/// Read send state.
pub async fn get_send_state(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path((mix_index, channel_index)): Path<(usize, usize)>,
) -> Result<Json<SendState>, ApiError> {
    validate_indexes(mix_index, channel_index)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    check_mix_ownership(&claims, mix_index, &state.db)?;
    Ok(Json(read_state(&state, mix_index, channel_index)?))
}
/// Set send gain.
pub async fn set_send_gain(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path((mix_index, channel_index)): Path<(usize, usize)>,
    Json(body): Json<GainRequest>,
) -> Result<Json<SendState>, ApiError> {
    validate_indexes(mix_index, channel_index)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    check_mix_ownership(&claims, mix_index, &state.db)?;
    if !body.gain_db.is_finite() || !(GAIN_DB_MIN..=GAIN_DB_MAX).contains(&body.gain_db) {
        return Err(ApiError::BadRequest("gain_db out of range".to_owned()));
    }
    {
        let mut ctrl = state
            .control
            .lock()
            .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
        ctrl.mix_mut(mix_index)
            .ok_or_else(|| ApiError::NotFound("mix not configured".to_owned()))?
            .set_send_gain_db(channel_index, body.gain_db)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    }
    Ok(Json(read_state(&state, mix_index, channel_index)?))
}
/// Set send pan.
pub async fn set_send_pan(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path((mix_index, channel_index)): Path<(usize, usize)>,
    Json(body): Json<PanRequest>,
) -> Result<Json<SendState>, ApiError> {
    validate_indexes(mix_index, channel_index)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    check_mix_ownership(&claims, mix_index, &state.db)?;
    if !body.pan.is_finite() || !(-1.0..=1.0).contains(&body.pan) {
        return Err(ApiError::BadRequest("pan out of range".to_owned()));
    }
    {
        let mut ctrl = state
            .control
            .lock()
            .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
        ctrl.mix_mut(mix_index)
            .ok_or_else(|| ApiError::NotFound("mix not configured".to_owned()))?
            .set_send_pan(channel_index, body.pan)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    }
    Ok(Json(read_state(&state, mix_index, channel_index)?))
}
/// Set send mute.
pub async fn set_send_muted(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path((mix_index, channel_index)): Path<(usize, usize)>,
    Json(body): Json<MuteRequest>,
) -> Result<Json<SendState>, ApiError> {
    validate_indexes(mix_index, channel_index)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    check_mix_ownership(&claims, mix_index, &state.db)?;
    {
        let mut ctrl = state
            .control
            .lock()
            .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
        ctrl.mix_mut(mix_index)
            .ok_or_else(|| ApiError::NotFound("mix not configured".to_owned()))?
            .set_send_muted(channel_index, body.muted)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    }
    Ok(Json(read_state(&state, mix_index, channel_index)?))
}
