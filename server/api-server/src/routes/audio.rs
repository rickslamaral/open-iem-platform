//! WebRTC audio transport signaling routes.

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};
use axum::{extract::State, response::IntoResponse, Json};
use control_protocol::Role;
use mix_engine::MAX_MIXES;
use serde::{Deserialize, Serialize};
use streaming::SessionInfo;

const MAX_MIX_ID_BYTES: usize = 128;

fn validate_mix_id(
    mix_id: Option<&str>,
    claims: &JwtClaims,
    state: &AppState,
) -> Result<(), ApiError> {
    let Some(value) = mix_id else {
        return Ok(());
    };
    if value.is_empty() || value.len() > MAX_MIX_ID_BYTES {
        return Err(ApiError::BadRequest("mix_id has invalid length".to_owned()));
    }
    if claims.role == Role::Musician {
        let mix_index = value
            .parse::<usize>()
            .map_err(|_| ApiError::BadRequest("mix_id must be a numeric mix index".to_owned()))?;
        if mix_index >= MAX_MIXES
            || state.db.get_user_assigned_mix(claims.user_id)? != Some(mix_index)
        {
            return Err(ApiError::Forbidden("musician does not own this mix"));
        }
    }
    Ok(())
}

/// SDP offer request.
#[derive(Debug, Deserialize)]
pub struct OfferRequest {
    /// Browser-generated SDP offer.
    pub sdp: String,
    /// Optional server-side mix assignment.
    pub mix_id: Option<String>,
}

/// SDP answer response.
#[derive(Debug, Serialize)]
pub struct OfferResponse {
    /// Server-generated SDP answer.
    pub sdp: String,
}

/// ICE candidate request.
#[derive(Debug, Deserialize)]
pub struct IceCandidateRequest {
    /// Browser-generated candidate string.
    pub candidate: String,
}

/// Active audio session response.
#[derive(Debug, Serialize)]
pub struct SessionsResponse {
    /// Active sessions.
    pub sessions: Vec<SessionInfo>,
}

/// `POST /api/v1/audio/offer` — negotiate one audio session for authenticated user.
///
/// # Errors
/// Returns `ApiError::Forbidden` for roles below Musician and `ApiError::BadRequest`
/// when SDP or input bounds fail.
pub async fn offer(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Json(body): Json<OfferRequest>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Musician)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    validate_mix_id(body.mix_id.as_deref(), &claims, &state)?;
    let answer = state
        .streaming
        .negotiate_offer(&claims.sub, &body.sdp, body.mix_id)
        .await
        .map_err(|_| ApiError::BadRequest("invalid SDP offer".to_owned()))?;
    Ok(Json(OfferResponse { sdp: answer }))
}

/// `POST /api/v1/audio/ice-candidate` — add candidate to caller's session.
///
/// # Errors
/// Returns `ApiError::Forbidden` for roles below Musician and `ApiError::BadRequest`
/// for malformed candidates or missing sessions.
pub async fn ice_candidate(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Json(body): Json<IceCandidateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Musician)?;
    state
        .streaming
        .add_ice_candidate(&claims.sub, &body.candidate)
        .await
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    Ok(Json(serde_json::json!({ "accepted": true })))
}

/// `GET /api/v1/audio/sessions` — list active audio sessions.
///
/// # Errors
/// Returns `ApiError::Forbidden` unless caller has Engineer role.
pub async fn sessions(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    Ok(Json(SessionsResponse {
        sessions: state.streaming.list().await,
    }))
}
