//! Built-in preset catalog and bounded channel preset application.

use axum::{
    extract::{Path, State},
    Json,
};
use control_protocol::{ClientMessage, Envelope, Role, ServerMessage};
use mix_engine::MAX_CHANNELS;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};

/// Immutable preset catalog entry.
#[derive(Debug, Serialize)]
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
        presets: vec![
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
        ],
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
    if !matches!(preset_id.as_str(), "default-vocal" | "default-instrument") {
        return Err(ApiError::BadRequest("unknown preset".to_owned()));
    }
    if usize::from(body.channel_index) >= MAX_CHANNELS {
        return Err(ApiError::BadRequest("channel is not available".to_owned()));
    }

    // Both built-ins intentionally represent safe neutral channel defaults.
    // Dispatch validates slot bounds before any mutation and each valid
    // dispatch is infallible after that validation.
    let mut ctrl = state
        .control
        .lock()
        .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
    let gain = ctrl.dispatch(Envelope::new(
        Uuid::new_v4().to_string(),
        ClientMessage::SetChannelGain {
            channel: body.channel_index,
            gain_db: 0.0,
        },
    ));
    if !matches!(gain.payload, ServerMessage::State { .. }) {
        return Err(ApiError::BadRequest("channel is not available".to_owned()));
    }
    let mute = ctrl.dispatch(Envelope::new(
        Uuid::new_v4().to_string(),
        ClientMessage::SetChannelMute {
            channel: body.channel_index,
            muted: false,
        },
    ));
    let ServerMessage::State { revision } = mute.payload else {
        return Err(ApiError::BadRequest("channel is not available".to_owned()));
    };
    Ok(Json(serde_json::json!({
        "preset_id": preset_id,
        "channel_index": body.channel_index,
        "revision": revision,
        "applied": true,
    })))
}
