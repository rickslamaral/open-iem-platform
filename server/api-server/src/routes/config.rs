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
