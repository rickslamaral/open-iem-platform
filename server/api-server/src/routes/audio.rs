//! WebRTC audio transport signaling routes.

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use control_protocol::Role;
use mix_engine::MAX_MIXES;
use serde::{Deserialize, Serialize};
use streaming::{PairingError, SessionInfo};

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
    /// Optional device ID for pairing-based authentication.
    pub device_id: Option<String>,
    /// Optional base64-encoded device credential.
    pub credential: Option<String>,
}

/// Pair device request body.
#[derive(Debug, Deserialize)]
pub struct PairDeviceRequest {
    /// Unique device identifier.
    pub device_id: String,
    /// Musician user identifier to bind this device to.
    pub musician_id: String,
    /// Mix slot index this device is authorized for.
    pub mix_index: usize,
    /// Base64-encoded device credential.
    pub credential: String,
}

/// Pair device response.
#[derive(Debug, Serialize)]
pub struct PairDeviceResponse {
    /// Paired device identifier.
    pub device_id: String,
    /// Mix slot index the device was paired to.
    pub mix_index: usize,
}

/// Revoke device response.
#[derive(Debug, Serialize)]
pub struct RevokeDeviceResponse {
    /// Whether the device was successfully revoked.
    pub revoked: bool,
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
    // Pairing fields are an inseparable credential boundary. Do not silently
    // downgrade a partially supplied pairing attempt to legacy auth.
    match (&body.device_id, &body.credential) {
        (Some(_), None) | (None, Some(_)) => {
            return Err(ApiError::BadRequest(
                "device_id and credential must be provided together".to_owned(),
            ));
        }
        _ => {}
    }
    // Device pairing authentication (optional, backward-compatible).
    if let (Some(ref device_id), Some(ref cred_str)) = (&body.device_id, &body.credential) {
        let cred_bytes = general_purpose::STANDARD
            .decode(cred_str)
            .map_err(|_| ApiError::BadRequest("credential is not valid base64".to_owned()))?;
        let identity = state
            .pairing
            .authenticate(device_id, &cred_bytes)
            .await
            .map_err(|e| match e {
                PairingError::Revoked => ApiError::Forbidden("device has been revoked"),
                _ => ApiError::Unauthorized("device authentication failed"),
            })?;
        if let Some(ref mix_id_str) = body.mix_id {
            let requested_mix: usize = mix_id_str.parse().map_err(|_| {
                ApiError::BadRequest("mix_id must be a numeric mix index".to_owned())
            })?;
            if identity.mix_index != requested_mix {
                return Err(ApiError::Forbidden("device is not authorized for this mix"));
            }
        }
    }
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

/// `POST /api/v1/audio/pairing` — pair a device to a musician/mix. Engineer/Admin only.
///
/// # Errors
/// Returns appropriate `ApiError` variants for invalid identity, duplicate devices, etc.
pub async fn pair_device(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Json(body): Json<PairDeviceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let cred_bytes = general_purpose::STANDARD
        .decode(&body.credential)
        .map_err(|_| ApiError::BadRequest("credential is not valid base64".to_owned()))?;
    let identity = state
        .pairing
        .pair(
            &body.device_id,
            &body.musician_id,
            body.mix_index,
            &cred_bytes,
        )
        .await
        .map_err(|e| match e {
            PairingError::InvalidIdentity => {
                ApiError::BadRequest("invalid device identity".to_owned())
            }
            PairingError::AlreadyPaired => {
                ApiError::Conflict("device is already paired".to_owned())
            }
            PairingError::InvalidCredential => {
                ApiError::BadRequest("invalid credential".to_owned())
            }
            PairingError::CapacityReached => {
                ApiError::Internal("pairing capacity reached".to_owned())
            }
            PairingError::NotFound => ApiError::NotFound("device not found".to_owned()),
            PairingError::Revoked => ApiError::BadRequest("device is revoked".to_owned()),
        })?;
    Ok(Json(PairDeviceResponse {
        device_id: identity.device_id,
        mix_index: identity.mix_index,
    }))
}

/// `DELETE /api/v1/audio/pairing/:device_id` — revoke device pairing. Engineer/Admin only.
///
/// # Errors
/// Returns `ApiError::NotFound` when the device is not registered.
pub async fn revoke_device(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    state
        .pairing
        .revoke(&device_id)
        .await
        .map_err(|e| match e {
            PairingError::NotFound => ApiError::NotFound(format!("device {device_id} not found")),
            _ => ApiError::Internal("revoke failed".to_owned()),
        })?;
    Ok(Json(RevokeDeviceResponse { revoked: true }))
}
