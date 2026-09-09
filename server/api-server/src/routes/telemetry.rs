//! Control-plane telemetry contract.

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role};
use axum::{response::IntoResponse, Json};
use control_protocol::Role;
use serde::Serialize;

/// Versioned telemetry response. Unknown audio metrics stay `null`.
#[derive(Serialize)]
pub struct TelemetryResponse {
    /// Contract schema version.
    pub schema_version: u8,
    /// Backend availability.
    pub availability: &'static str,
    /// Backend identifier.
    pub backend: &'static str,
    /// Sample rate, unavailable until audio backend is connected.
    pub sample_rate_hz: Option<u32>,
    /// Processed frames, unavailable until audio backend is connected.
    pub frames_processed: Option<u64>,
    /// XRUN count, unavailable until audio backend is connected.
    pub xrun_count: Option<u64>,
}

/// `GET /api/v1/telemetry` — returns honest backend availability.
///
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Engineer or Admin role.
#[allow(clippy::unused_async)]
pub async fn get_telemetry(
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    Ok(Json(TelemetryResponse {
        schema_version: 1,
        availability: "simulated",
        backend: "simulated",
        sample_rate_hz: None,
        frames_processed: None,
        xrun_count: None,
    }))
}
