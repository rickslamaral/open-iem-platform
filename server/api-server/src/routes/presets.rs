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
