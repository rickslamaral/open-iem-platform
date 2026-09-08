//! `GET /api/v1/health` — liveness probe.

#![allow(clippy::unused_async)]

use axum::{response::IntoResponse, Json};
use serde::Serialize;

/// Health response body.
#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

/// Health check handler — no auth required.
pub async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}
