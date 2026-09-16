//! Observability metrics REST handler.

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};
use axum::{extract::State, response::IntoResponse, Json};
use control_protocol::Role;

/// `GET /api/v1/metrics` — returns bounded observability metrics snapshot.
///
/// Requires at least Engineer role. Counters are initialised to zero at startup
/// and increment as runtime events occur; all values are `SIMULATED` until
/// real audio backend and receiver are connected.
///
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Engineer or Admin role.
#[allow(clippy::unused_async)]
pub async fn get_metrics(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    Ok(Json(state.metrics.snapshot()))
}
