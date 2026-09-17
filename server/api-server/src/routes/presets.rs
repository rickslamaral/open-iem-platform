//! Read-only preset catalog. Mutation and application remain future work.

use axum::{extract::State, Json};
use control_protocol::Role;
use serde::Serialize;

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

/// GET /api/v1/presets — catalog visible to Engineer/Admin.
///
/// Built-in entries are deliberately immutable.
///
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Engineer role. Persistence and apply routes
/// require a separate schema and authorization design.
#[allow(clippy::unused_async)]
pub async fn list_presets(
    State(_state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<Json<PresetsResponse>, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
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
