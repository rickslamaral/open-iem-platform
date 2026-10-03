//! Band catalog and management routes.
#![allow(missing_docs, clippy::missing_errors_doc, clippy::unused_async)]
use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use control_protocol::Role;
use serde::{Deserialize, Serialize};
#[derive(Serialize)]
pub struct PublicBand {
    pub id: i64,
    pub name: String,
}
#[derive(Serialize)]
pub struct Band {
    pub id: i64,
    pub name: String,
    pub active: bool,
}
#[derive(Deserialize)]
pub struct BandInput {
    pub name: String,
    pub active: bool,
}
pub async fn list_public(State(state): State<AppState>) -> Result<Json<Vec<PublicBand>>, ApiError> {
    Ok(Json(
        state
            .db
            .list_active_bands()?
            .into_iter()
            .map(|(id, name)| PublicBand { id, name })
            .collect(),
    ))
}
pub async fn list(
    State(state): State<AppState>,
    Extension(c): Extension<JwtClaims>,
) -> Result<Json<Vec<Band>>, ApiError> {
    require_min_role(&c, Role::Engineer)?;
    Ok(Json(
        state
            .db
            .list_bands()?
            .into_iter()
            .map(|(id, name, active)| Band { id, name, active })
            .collect(),
    ))
}
pub async fn create(
    State(state): State<AppState>,
    Extension(c): Extension<JwtClaims>,
    Json(b): Json<BandInput>,
) -> Result<(StatusCode, Json<Band>), ApiError> {
    require_min_role(&c, Role::Engineer)?;
    let id = state.db.create_band(&b.name, b.active)?;
    Ok((
        StatusCode::CREATED,
        Json(Band {
            id,
            name: b.name.trim().to_owned(),
            active: b.active,
        }),
    ))
}
pub async fn update(
    State(state): State<AppState>,
    Extension(c): Extension<JwtClaims>,
    Path(id): Path<i64>,
    Json(b): Json<BandInput>,
) -> Result<Json<Band>, ApiError> {
    require_min_role(&c, Role::Engineer)?;
    state.db.update_band(id, &b.name, b.active)?;
    Ok(Json(Band {
        id,
        name: b.name.trim().to_owned(),
        active: b.active,
    }))
}
pub async fn delete(
    State(state): State<AppState>,
    Extension(c): Extension<JwtClaims>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    require_min_role(&c, Role::Engineer)?;
    state.db.delete_band(id)?;
    Ok(StatusCode::NO_CONTENT)
}
