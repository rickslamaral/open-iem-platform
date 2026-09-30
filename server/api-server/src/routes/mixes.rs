//! Mix assignment and musician-owned send routes.

#![allow(clippy::missing_errors_doc, clippy::unused_async)]

use crate::{
    auth::JwtClaims, db::Db, error::ApiError, middleware::require_min_role, state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use control_protocol::Role;
use mix_engine::{GAIN_DB_MAX, GAIN_DB_MIN, MAX_CHANNELS, MAX_MIXES};
use serde::{Deserialize, Serialize};

/// Mix assignment response.
#[derive(Serialize)]
pub struct MixAssignment {
    /// Mix slot.
    pub mix_index: usize,
    /// Assigned user ID.
    pub user_id: i64,
    /// Assigned username.
    pub username: String,
}
/// Assignment request.
#[derive(Deserialize)]
pub struct AssignRequest {
    /// User ID.
    pub user_id: i64,
}
/// Gain update request.
#[derive(Deserialize)]
pub struct GainRequest {
    /// Gain in dBFS.
    pub gain_db: f32,
}
/// Pan update request.
#[derive(Deserialize)]
pub struct PanRequest {
    /// Pan from -1 to 1.
    pub pan: f32,
}
/// Mute update request.
#[derive(Deserialize)]
pub struct MuteRequest {
    /// Mute state.
    pub muted: bool,
}
/// Send state response.
#[derive(Serialize)]
pub struct SendState {
    /// Mix slot.
    pub mix_index: usize,
    /// Channel slot.
    pub channel_index: usize,
    /// Gain in dBFS.
    pub gain_db: f32,
    /// Pan.
    pub pan: f32,
    /// Mute state.
    pub muted: bool,
    /// Send revision.
    pub revision: u64,
}

fn check_mix_ownership(claims: &JwtClaims, mix_index: usize, db: &Db) -> Result<(), ApiError> {
    if matches!(claims.role, Role::Admin | Role::Engineer) {
        return Ok(());
    }
    if db.get_user_assigned_mix(claims.user_id)? == Some(mix_index) {
        Ok(())
    } else {
        Err(ApiError::Forbidden("musician does not own this mix"))
    }
}
fn validate_indexes(mix_index: usize, channel_index: usize) -> Result<(), ApiError> {
    if mix_index >= MAX_MIXES || channel_index >= MAX_CHANNELS {
        Err(ApiError::BadRequest("index out of range".to_owned()))
    } else {
        Ok(())
    }
}

/// List assignments.
pub async fn list_mixes(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<Json<Vec<MixAssignment>>, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    Ok(Json(
        state
            .db
            .list_mix_assignments()?
            .into_iter()
            .map(|(mix_index, user_id, username)| MixAssignment {
                mix_index,
                user_id,
                username,
            })
            .collect(),
    ))
}
/// Assign user to mix.
pub async fn assign_mix(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(index): Path<usize>,
    Json(body): Json<AssignRequest>,
) -> Result<Json<MixAssignment>, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    if index >= MAX_MIXES || body.user_id <= 0 {
        return Err(ApiError::BadRequest("invalid mix assignment".to_owned()));
    }
    state.db.assign_mix(index, body.user_id)?;
    let username = state.db.find_user_by_id(body.user_id)?.0;
    Ok(Json(MixAssignment {
        mix_index: index,
        user_id: body.user_id,
        username,
    }))
}
/// Unassign mix.
pub async fn unassign_mix(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(index): Path<usize>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    state.db.unassign_mix(index)?;
    Ok(StatusCode::NO_CONTENT)
}
fn read_state(
    state: &AppState,
    mix_index: usize,
    channel_index: usize,
) -> Result<SendState, ApiError> {
    let ctrl = state
        .control
        .lock()
        .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
    let send = ctrl
        .mix(mix_index)
        .and_then(|mix| mix.get_send(channel_index));
    Ok(match send {
        Some(s) => SendState {
            mix_index,
            channel_index,
            gain_db: s.gain_db(),
            pan: s.pan(),
            muted: s.muted,
            revision: s.revision(),
        },
        None => SendState {
            mix_index,
            channel_index,
            gain_db: 0.0,
            pan: 0.0,
            muted: false,
            revision: 0,
        },
    })
}
/// Read send state.
pub async fn get_send_state(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path((mix_index, channel_index)): Path<(usize, usize)>,
) -> Result<Json<SendState>, ApiError> {
    validate_indexes(mix_index, channel_index)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    check_mix_ownership(&claims, mix_index, &state.db)?;
    Ok(Json(read_state(&state, mix_index, channel_index)?))
}
/// Set send gain.
pub async fn set_send_gain(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path((mix_index, channel_index)): Path<(usize, usize)>,
    Json(body): Json<GainRequest>,
) -> Result<Json<SendState>, ApiError> {
    validate_indexes(mix_index, channel_index)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    check_mix_ownership(&claims, mix_index, &state.db)?;
    if !body.gain_db.is_finite() || !(GAIN_DB_MIN..=GAIN_DB_MAX).contains(&body.gain_db) {
        return Err(ApiError::BadRequest("gain_db out of range".to_owned()));
    }
    {
        let mut ctrl = state
            .control
            .lock()
            .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
        ctrl.mix_mut(mix_index)
            .ok_or_else(|| ApiError::NotFound("mix not configured".to_owned()))?
            .set_send_gain_db(channel_index, body.gain_db)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    }
    Ok(Json(read_state(&state, mix_index, channel_index)?))
}
/// Set send pan.
pub async fn set_send_pan(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path((mix_index, channel_index)): Path<(usize, usize)>,
    Json(body): Json<PanRequest>,
) -> Result<Json<SendState>, ApiError> {
    validate_indexes(mix_index, channel_index)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    check_mix_ownership(&claims, mix_index, &state.db)?;
    if !body.pan.is_finite() || !(-1.0..=1.0).contains(&body.pan) {
        return Err(ApiError::BadRequest("pan out of range".to_owned()));
    }
    {
        let mut ctrl = state
            .control
            .lock()
            .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
        ctrl.mix_mut(mix_index)
            .ok_or_else(|| ApiError::NotFound("mix not configured".to_owned()))?
            .set_send_pan(channel_index, body.pan)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    }
    Ok(Json(read_state(&state, mix_index, channel_index)?))
}
/// Set send mute.
pub async fn set_send_muted(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path((mix_index, channel_index)): Path<(usize, usize)>,
    Json(body): Json<MuteRequest>,
) -> Result<Json<SendState>, ApiError> {
    validate_indexes(mix_index, channel_index)?;
    let _assignment_guard = state.mix_assignment_lock.lock().await;
    check_mix_ownership(&claims, mix_index, &state.db)?;
    {
        let mut ctrl = state
            .control
            .lock()
            .map_err(|_| ApiError::Internal("lock poisoned".to_owned()))?;
        ctrl.mix_mut(mix_index)
            .ok_or_else(|| ApiError::NotFound("mix not configured".to_owned()))?
            .set_send_muted(channel_index, body.muted)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    }
    Ok(Json(read_state(&state, mix_index, channel_index)?))
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{generate_refresh_token, hash_password, token_to_storage_key, JwtKeys},
        db::Db,
        middleware::jwt_auth,
        routes::mixes::{
            assign_mix, get_send_state, list_mixes, set_send_gain, set_send_muted, set_send_pan,
            unassign_mix,
        },
        security::validate_origin,
        state::AppState,
    };
    use axum::{
        middleware,
        routing::{get, post, put},
        Router,
    };
    use axum_test::TestServer;
    use control_protocol::Role;
    use control_server::ControlState;
    use mix_engine::Mix;
    use serde_json::json;
    use std::{fs, process::Command};
    use uuid::Uuid;

    fn test_keys() -> (Vec<u8>, Vec<u8>) {
        let nonce = Uuid::new_v4();
        let private_path = std::env::temp_dir().join(format!("open-iem-mixes-test-{nonce}.pem"));
        let public_path = std::env::temp_dir().join(format!("open-iem-mixes-test-{nonce}.pub.pem"));
        let _ = fs::remove_file(&private_path);
        let _ = fs::remove_file(&public_path);
        Command::new("openssl")
            .args(["genpkey", "-algorithm", "ed25519", "-out"])
            .arg(&private_path)
            .status()
            .expect("openssl must be installed")
            .success()
            .then_some(())
            .expect("openssl key generation must succeed");
        Command::new("openssl")
            .args(["pkey", "-in"])
            .arg(&private_path)
            .args(["-pubout", "-out"])
            .arg(&public_path)
            .status()
            .expect("openssl public-key export")
            .success()
            .then_some(())
            .expect("openssl public-key export must succeed");
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
        let mut control = ControlState::new();
        control.set_mix(0, Mix::new(1, "test")).expect("seed mix 0");
        let state = AppState::new(control, db, jwt);

        let protected = Router::new()
            .route("/api/v1/mixes", get(list_mixes))
            .route(
                "/api/v1/mixes/{index}/assign",
                post(assign_mix).delete(unassign_mix),
            )
            .route(
                "/api/v1/mixes/{mix_idx}/sends/{ch_idx}",
                get(get_send_state),
            )
            .route(
                "/api/v1/mixes/{mix_idx}/sends/{ch_idx}/gain",
                put(set_send_gain),
            )
            .route(
                "/api/v1/mixes/{mix_idx}/sends/{ch_idx}/pan",
                put(set_send_pan),
            )
            .route(
                "/api/v1/mixes/{mix_idx}/sends/{ch_idx}/mute",
                put(set_send_muted),
            )
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

    #[tokio::test]
    async fn list_mixes_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng1", Role::Engineer);
        let resp = server
            .get("/api/v1/mixes")
            .authorization_bearer(token)
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert!(body.is_array());
    }

    #[tokio::test]
    async fn list_mixes_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus1", Role::Musician);
        let resp = server
            .get("/api/v1/mixes")
            .authorization_bearer(token)
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn list_mixes_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server.get("/api/v1/mixes").await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn assign_mix_engineer_ok() {
        let (server, state) = build_test_app();
        let eng_token = seed_user_token(&state, "eng2", Role::Engineer);
        // Create a Musician to assign.
        let pw_hash = hash_password("pass").expect("hash");
        state
            .db
            .create_user("mus_a", &pw_hash, Role::Musician)
            .expect("create musician");
        let (user_id, _, _, _) = state.db.find_user("mus_a").expect("user exists");
        let resp = server
            .post("/api/v1/mixes/0/assign")
            .authorization_bearer(eng_token)
            .json(&json!({ "user_id": user_id }))
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["mix_index"], 0);
        assert_eq!(body["user_id"], user_id);
    }

    #[tokio::test]
    async fn assign_mix_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus2", Role::Musician);
        let (user_id, _, _, _) = state.db.find_user("mus2").expect("user exists");
        let resp = server
            .post("/api/v1/mixes/0/assign")
            .authorization_bearer(token)
            .json(&json!({ "user_id": user_id }))
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn assign_mix_out_of_range_index() {
        let (server, state) = build_test_app();
        let eng_token = seed_user_token(&state, "eng3", Role::Engineer);
        let pw_hash = hash_password("pass").expect("hash");
        state
            .db
            .create_user("mus_b", &pw_hash, Role::Musician)
            .expect("create musician");
        let (user_id, _, _, _) = state.db.find_user("mus_b").expect("user exists");
        // mix index 99 >= MAX_MIXES(2), handler returns 400
        let resp = server
            .post("/api/v1/mixes/99/assign")
            .authorization_bearer(eng_token)
            .json(&json!({ "user_id": user_id }))
            .await;
        assert!(
            resp.status_code().is_client_error(),
            "out-of-range mix index must be a client error, got {}",
            resp.status_code()
        );
    }

    #[tokio::test]
    async fn unassign_mix_engineer_ok() {
        let (server, state) = build_test_app();
        let eng_token = seed_user_token(&state, "eng4", Role::Engineer);
        let pw_hash = hash_password("pass").expect("hash");
        state
            .db
            .create_user("mus_c", &pw_hash, Role::Musician)
            .expect("create musician");
        let (user_id, _, _, _) = state.db.find_user("mus_c").expect("user exists");
        state.db.assign_mix(0, user_id).expect("assign");
        let resp = server
            .delete("/api/v1/mixes/0/assign")
            .authorization_bearer(eng_token)
            .await;
        resp.assert_status(axum::http::StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn unassign_mix_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus3", Role::Musician);
        let resp = server
            .delete("/api/v1/mixes/0/assign")
            .authorization_bearer(token)
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn get_send_state_musician_owns_mix() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus4", Role::Musician);
        let (user_id, _, _, _) = state.db.find_user("mus4").expect("user exists");
        state.db.assign_mix(0, user_id).expect("assign mix");
        let resp = server
            .get("/api/v1/mixes/0/sends/0")
            .authorization_bearer(token)
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["mix_index"], 0);
        assert_eq!(body["channel_index"], 0);
    }

    #[tokio::test]
    async fn get_send_state_musician_wrong_mix() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus5", Role::Musician);
        let (user_id, _, _, _) = state.db.find_user("mus5").expect("user exists");
        // Assign to mix 0; attempt to access mix 1
        state.db.assign_mix(0, user_id).expect("assign mix");
        let resp = server
            .get("/api/v1/mixes/1/sends/0")
            .authorization_bearer(token)
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn get_send_state_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server.get("/api/v1/mixes/0/sends/0").await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn set_send_gain_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng5", Role::Engineer);
        let resp = server
            .put("/api/v1/mixes/0/sends/0/gain")
            .authorization_bearer(token)
            .json(&json!({ "gain_db": 0.0_f32 }))
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert!(body["revision"].is_number());
    }

    #[tokio::test]
    async fn set_send_gain_out_of_range() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng6", Role::Engineer);
        let resp = server
            .put("/api/v1/mixes/0/sends/0/gain")
            .authorization_bearer(token)
            .json(&json!({ "gain_db": 200.0_f32 }))
            .await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn set_send_pan_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng7", Role::Engineer);
        let resp = server
            .put("/api/v1/mixes/0/sends/0/pan")
            .authorization_bearer(token)
            .json(&json!({ "pan": 0.0_f32 }))
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert!(body["revision"].is_number());
    }

    #[tokio::test]
    async fn set_send_pan_out_of_range() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng8", Role::Engineer);
        let resp = server
            .put("/api/v1/mixes/0/sends/0/pan")
            .authorization_bearer(token)
            .json(&json!({ "pan": 5.0_f32 }))
            .await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn set_send_muted_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "eng9", Role::Engineer);
        let resp = server
            .put("/api/v1/mixes/0/sends/0/mute")
            .authorization_bearer(token)
            .json(&json!({ "muted": true }))
            .await;
        resp.assert_status_success();
        let body: serde_json::Value = resp.json();
        assert!(body["revision"].is_number());
    }

    #[tokio::test]
    async fn set_send_muted_musician_wrong_mix() {
        let (server, state) = build_test_app();
        let token = seed_user_token(&state, "mus6", Role::Musician);
        let (user_id, _, _, _) = state.db.find_user("mus6").expect("user exists");
        // Assign to mix 0, attempt to mute on mix 1
        state.db.assign_mix(0, user_id).expect("assign mix");
        let resp = server
            .put("/api/v1/mixes/1/sends/0/mute")
            .authorization_bearer(token)
            .json(&json!({ "muted": false }))
            .await;
        resp.assert_status_forbidden();
    }
}
