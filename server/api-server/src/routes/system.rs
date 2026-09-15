//! Public system information REST handler.

use axum::Json;
use control_protocol::PROTOCOL_VERSION;
use mix_engine::{MAX_CHANNELS, MAX_MIXES};
use serde::Serialize;

/// Public API and backend capability information.
#[derive(Serialize)]
pub struct SystemInfo {
    /// Contract schema version.
    pub schema_version: u8,
    /// API server package version.
    pub version: &'static str,
    /// Backend runtime status.
    pub backend_status: &'static str,
    /// Maximum supported input channels.
    pub max_channels: usize,
    /// Maximum supported mixes.
    pub max_mixes: usize,
    /// Control protocol version.
    pub protocol_version: u16,
}

/// `GET /api/v1/system` — returns public system information.
#[allow(clippy::unused_async)]
pub async fn get_system_info() -> Json<SystemInfo> {
    Json(SystemInfo {
        schema_version: 1,
        version: env!("CARGO_PKG_VERSION"),
        backend_status: "SIMULATED",
        max_channels: MAX_CHANNELS,
        max_mixes: MAX_MIXES,
        protocol_version: PROTOCOL_VERSION,
    })
}
