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
use streaming::{
    PairingError, SessionInfo, MAX_CANDIDATE_BYTES, MAX_CREDENTIAL_BYTES, MAX_MIX_ID_BYTES,
    MAX_SDP_BYTES, MAX_USER_ID_BYTES,
};

const MAX_DTLS_FINGERPRINT_BYTES: usize = 103;

fn validate_field_len(name: &str, value: &str, max: usize) -> Result<(), ApiError> {
    if value.len() > max {
        return Err(ApiError::BadRequest(format!(
            "{name} exceeds maximum length"
        )));
    }
    Ok(())
}

fn validate_optional_field_len(
    name: &str,
    value: Option<&str>,
    max: usize,
) -> Result<(), ApiError> {
    if let Some(value) = value {
        validate_field_len(name, value, max)?;
    }
    Ok(())
}

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
    /// Optional canonical DTLS fingerprint enrolled during pairing.
    pub dtls_fingerprint: Option<String>,
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
    validate_field_len("sdp", &body.sdp, MAX_SDP_BYTES)?;
    validate_optional_field_len("mix_id", body.mix_id.as_deref(), MAX_MIX_ID_BYTES)?;
    validate_optional_field_len("device_id", body.device_id.as_deref(), MAX_USER_ID_BYTES)?;
    validate_optional_field_len(
        "credential",
        body.credential.as_deref(),
        MAX_CREDENTIAL_BYTES,
    )?;
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
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    // Device pairing authentication (optional, backward-compatible).
    let mut authenticated_identity = None;
    let mut session_mix_id = body.mix_id.clone();
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
        session_mix_id = Some(identity.mix_index.to_string());
        authenticated_identity = Some(identity);
    }
    validate_mix_id(session_mix_id.as_deref(), &claims, &state)?;
    let answer = state
        .streaming
        .negotiate_offer_bound(
            &claims.sub,
            &body.sdp,
            session_mix_id,
            authenticated_identity.as_ref(),
        )
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
    validate_field_len("candidate", &body.candidate, MAX_CANDIDATE_BYTES)?;
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
    validate_field_len("device_id", &body.device_id, MAX_USER_ID_BYTES)?;
    validate_field_len("musician_id", &body.musician_id, MAX_USER_ID_BYTES)?;
    validate_field_len("credential", &body.credential, MAX_CREDENTIAL_BYTES)?;
    validate_optional_field_len(
        "dtls_fingerprint",
        body.dtls_fingerprint.as_deref(),
        MAX_DTLS_FINGERPRINT_BYTES,
    )?;
    let cred_bytes = general_purpose::STANDARD
        .decode(&body.credential)
        .map_err(|_| ApiError::BadRequest("credential is not valid base64".to_owned()))?;
    let identity = state
        .pairing
        .pair_with_fingerprint(
            &body.device_id,
            &body.musician_id,
            body.mix_index,
            &cred_bytes,
            body.dtls_fingerprint,
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
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    state
        .pairing
        .revoke(&device_id)
        .await
        .map_err(|e| match e {
            PairingError::NotFound => ApiError::NotFound(format!("device {device_id} not found")),
            _ => ApiError::Internal("revoke failed".to_owned()),
        })?;
    state.streaming.remove_by_device_id(&device_id).await;
    Ok(Json(RevokeDeviceResponse { revoked: true }))
}

/// Re-pair device request body.
#[derive(Debug, Deserialize)]
pub struct RepairDeviceRequest {
    /// Current (old) base64-encoded device credential.
    pub old_credential: String,
    /// New base64-encoded device credential.
    pub new_credential: String,
}

/// Re-pair device response.
#[derive(Debug, Serialize)]
pub struct RepairDeviceResponse {
    /// Device that was re-paired.
    pub device_id: String,
    /// Whether re-pairing succeeded.
    pub repaired: bool,
}

/// `PUT /api/v1/audio/pairing/:device_id` — re-pair a previously revoked device.
/// Engineer/Admin only. Requires old credential to authorize credential rotation.
///
/// # Errors
/// Returns `ApiError::NotFound` when device is unknown, `ApiError::Conflict` when
/// device is not revoked, and `ApiError::Unauthorized` on bad credentials.
pub async fn repair_device(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(device_id): Path<String>,
    Json(body): Json<RepairDeviceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    validate_field_len("device_id", &device_id, MAX_USER_ID_BYTES)?;
    validate_field_len("old_credential", &body.old_credential, MAX_CREDENTIAL_BYTES)?;
    validate_field_len("new_credential", &body.new_credential, MAX_CREDENTIAL_BYTES)?;
    let old_bytes = general_purpose::STANDARD
        .decode(&body.old_credential)
        .map_err(|_| ApiError::BadRequest("old_credential is not valid base64".to_owned()))?;
    let new_bytes = general_purpose::STANDARD
        .decode(&body.new_credential)
        .map_err(|_| ApiError::BadRequest("new_credential is not valid base64".to_owned()))?;
    state
        .pairing
        .replace_revoked(&device_id, &old_bytes, &new_bytes)
        .await
        .map_err(|e| match e {
            PairingError::NotFound => ApiError::NotFound(format!("device {device_id} not found")),
            PairingError::AlreadyPaired => ApiError::Conflict("device is not revoked".to_owned()),
            PairingError::InvalidCredential => ApiError::Unauthorized("invalid credential"),
            _ => ApiError::Internal("re-pair failed".to_owned()),
        })?;
    Ok(Json(RepairDeviceResponse {
        device_id,
        repaired: true,
    }))
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{generate_refresh_token, hash_password, token_to_storage_key, JwtKeys},
        db::Db,
        middleware::jwt_auth,
        routes::audio::{
            ice_candidate, offer, pair_device, repair_device, revoke_device, sessions,
        },
        security::validate_origin,
        state::AppState,
    };
    use axum::{
        middleware,
        routing::{delete, get, post, put},
        Router,
    };
    use axum_test::TestServer;
    use control_protocol::Role;
    use control_server::ControlState;
    use serde_json::json;
    use std::{
        fs,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn test_keys() -> (Vec<u8>, Vec<u8>) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be valid")
            .as_nanos();
        let private_path = std::env::temp_dir().join(format!("open-iem-audio-test-{nonce}.pem"));
        let public_path = std::env::temp_dir().join(format!("open-iem-audio-test-{nonce}.pub.pem"));
        let _ = fs::remove_file(&private_path);
        let _ = fs::remove_file(&public_path);
        let status = Command::new("openssl")
            .args(["genpkey", "-algorithm", "ed25519", "-out"])
            .arg(&private_path)
            .status()
            .expect("openssl must be installed for integration tests");
        assert!(status.success(), "openssl key generation failed");
        let status = Command::new("openssl")
            .args(["pkey", "-in"])
            .arg(&private_path)
            .args(["-pubout", "-out"])
            .arg(&public_path)
            .status()
            .expect("openssl public-key export");
        assert!(status.success(), "openssl public-key export failed");
        let private = fs::read(&private_path).expect("private key must be readable");
        let public = fs::read(&public_path).expect("public key must be readable");
        let _ = fs::remove_file(private_path);
        let _ = fs::remove_file(public_path);
        (private, public)
    }

    fn build_test_app() -> (TestServer, AppState) {
        let (private_pem, public_pem) = test_keys();
        let jwt = JwtKeys::from_ed_pem(&private_pem, &public_pem).expect("test PEM valid");
        let db = Db::open_in_memory().expect("in-memory DB");
        let state = AppState::new(ControlState::new(), db, jwt);

        let protected = Router::new()
            .route("/api/v1/audio/offer", post(offer))
            .route("/api/v1/audio/ice-candidate", post(ice_candidate))
            .route("/api/v1/audio/sessions", get(sessions))
            .route("/api/v1/audio/pairing", post(pair_device))
            .route("/api/v1/audio/pairing/{device_id}", delete(revoke_device))
            .route("/api/v1/audio/pairing/{device_id}", put(repair_device))
            .layer(middleware::from_fn_with_state(state.clone(), jwt_auth));

        let app = Router::new()
            .merge(protected)
            .with_state(state.clone())
            .layer(middleware::from_fn(validate_origin));

        (TestServer::new(app), state)
    }

    fn seed_user_token(state: &AppState, username: &str, role: Role) -> String {
        let pw_hash = hash_password("pass").expect("hash");
        state
            .db
            .create_user(username, &pw_hash, role)
            .expect("create user");
        let (user_id, _, _, _) = state.db.find_user(username).expect("user exists");
        let jti = uuid::Uuid::new_v4().to_string();
        let raw_refresh = generate_refresh_token();
        let session_id = state
            .db
            .create_session_with_access(
                user_id,
                &token_to_storage_key(&raw_refresh),
                4_000_000_000,
                &uuid::Uuid::new_v4().to_string(),
                &jti,
                4_000_000_000,
            )
            .expect("session created");
        state
            .jwt
            .issue_with_session(username, user_id, role, &jti, Some(session_id))
            .expect("token issued")
    }

    // --- offer ---

    #[tokio::test]
    async fn offer_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server
            .post("/api/v1/audio/offer")
            .json(&json!({ "sdp": "v=0\r\n" }))
            .await;
        assert_eq!(resp.status_code(), 401);
    }

    #[tokio::test]
    async fn offer_oversized_sdp_returns_400() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus_offer", Role::Musician);
        // 16 KiB + 1 byte
        let oversized = "x".repeat(16 * 1024 + 1);
        let resp = server
            .post("/api/v1/audio/offer")
            .add_header(
                axum::http::HeaderName::from_static("authorization"),
                axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            )
            .json(&json!({ "sdp": oversized }))
            .await;
        assert_eq!(resp.status_code(), 400);
    }

    #[tokio::test]
    async fn offer_partial_pairing_credentials_returns_400() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus_partial_cred", Role::Musician);
        // device_id without credential — inseparable boundary
        let resp = server
            .post("/api/v1/audio/offer")
            .add_header(
                axum::http::HeaderName::from_static("authorization"),
                axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            )
            .json(&json!({ "sdp": "v=0\r\n", "device_id": "dev1" }))
            .await;
        assert_eq!(resp.status_code(), 400);
    }

    // --- ice_candidate ---

    #[tokio::test]
    async fn ice_candidate_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server
            .post("/api/v1/audio/ice-candidate")
            .json(&json!({ "candidate": "candidate:0 1 UDP 2122252543 192.0.2.1 56000 typ host" }))
            .await;
        assert_eq!(resp.status_code(), 401);
    }

    #[tokio::test]
    async fn ice_candidate_oversized_candidate_returns_400() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus_ice", Role::Musician);
        // 2049 bytes > MAX_CANDIDATE_BYTES (2048)
        let oversized = "x".repeat(2049);
        let resp = server
            .post("/api/v1/audio/ice-candidate")
            .add_header(
                axum::http::HeaderName::from_static("authorization"),
                axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            )
            .json(&json!({ "candidate": oversized }))
            .await;
        assert_eq!(resp.status_code(), 400);
    }

    // --- sessions ---

    #[tokio::test]
    async fn sessions_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server.get("/api/v1/audio/sessions").await;
        assert_eq!(resp.status_code(), 401);
    }

    #[tokio::test]
    async fn sessions_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus_sessions", Role::Musician);
        let resp = server
            .get("/api/v1/audio/sessions")
            .add_header(
                axum::http::HeaderName::from_static("authorization"),
                axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            )
            .await;
        assert_eq!(resp.status_code(), 403);
    }

    #[tokio::test]
    async fn sessions_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng_sessions", Role::Engineer);
        let resp = server
            .get("/api/v1/audio/sessions")
            .add_header(
                axum::http::HeaderName::from_static("authorization"),
                axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            )
            .await;
        assert_eq!(resp.status_code(), 200);
    }

    // --- pair_device ---

    #[tokio::test]
    async fn pair_device_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server
            .post("/api/v1/audio/pairing")
            .json(&json!({
                "device_id": "dev1",
                "musician_id": "user1",
                "mix_index": 0,
                "credential": "dGVzdA=="
            }))
            .await;
        assert_eq!(resp.status_code(), 401);
    }

    #[tokio::test]
    async fn pair_device_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus_pair", Role::Musician);
        let resp = server
            .post("/api/v1/audio/pairing")
            .add_header(
                axum::http::HeaderName::from_static("authorization"),
                axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            )
            .json(&json!({
                "device_id": "dev1",
                "musician_id": "user1",
                "mix_index": 0,
                "credential": "dGVzdA=="
            }))
            .await;
        assert_eq!(resp.status_code(), 403);
    }

    #[tokio::test]
    async fn pair_device_invalid_base64_credential_returns_400() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng_pair_bad_b64", Role::Engineer);
        let resp = server
            .post("/api/v1/audio/pairing")
            .add_header(
                axum::http::HeaderName::from_static("authorization"),
                axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            )
            .json(&json!({
                "device_id": "dev1",
                "musician_id": "user1",
                "mix_index": 0,
                "credential": "not!valid!base64!!!"
            }))
            .await;
        assert_eq!(resp.status_code(), 400);
    }

    // --- revoke_device ---

    #[tokio::test]
    async fn revoke_device_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server.delete("/api/v1/audio/pairing/dev1").await;
        assert_eq!(resp.status_code(), 401);
    }

    #[tokio::test]
    async fn revoke_device_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus_revoke", Role::Musician);
        let resp = server
            .delete("/api/v1/audio/pairing/dev1")
            .add_header(
                axum::http::HeaderName::from_static("authorization"),
                axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            )
            .await;
        assert_eq!(resp.status_code(), 403);
    }

    #[tokio::test]
    async fn revoke_device_unknown_device_returns_404() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng_revoke_unknown", Role::Engineer);
        let resp = server
            .delete("/api/v1/audio/pairing/no-such-device")
            .add_header(
                axum::http::HeaderName::from_static("authorization"),
                axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            )
            .await;
        assert_eq!(resp.status_code(), 404);
    }

    // --- repair_device ---

    #[tokio::test]
    async fn repair_device_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server
            .put("/api/v1/audio/pairing/dev1")
            .json(&json!({ "old_credential": "dGVzdA==", "new_credential": "dGVzdA==" }))
            .await;
        assert_eq!(resp.status_code(), 401);
    }

    #[tokio::test]
    async fn repair_device_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus_repair", Role::Musician);
        let resp = server
            .put("/api/v1/audio/pairing/dev1")
            .add_header(
                axum::http::HeaderName::from_static("authorization"),
                axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            )
            .json(&json!({ "old_credential": "dGVzdA==", "new_credential": "dGVzdA==" }))
            .await;
        assert_eq!(resp.status_code(), 403);
    }
}
