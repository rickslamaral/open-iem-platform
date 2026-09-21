//! HTTP integration tests for api-server.
//!
//! Tests spin up a full Axum router with an in-memory SQLite database and
//! test-only Ed25519 keys. Every test exercises the complete middleware stack:
//! Origin/CSRF check → JWT auth → role check → handler.
//!
//! Test keys are generated at compile time from a fixed PEM pair that carries
//! no sensitive information (test-only, never used for real tokens).

use api_server::{
    auth::{generate_refresh_token, hash_password, token_to_storage_key, JwtKeys},
    db::Db,
    middleware::jwt_auth,
    routes::{
        admin::{
            delete_user as admin_delete_user, list_sessions as admin_list_sessions,
            list_users as admin_list_users, revoke_session as admin_revoke_session,
        },
        audio::{ice_candidate, offer, pair_device, revoke_device, sessions},
        auth::{create_user, login, logout, refresh},
        channels::{get_state, list_channels, set_channel_gain, set_channel_mute},
        health::health,
        metrics::get_metrics,
        mixes::{
            assign_mix, get_send_state, list_mixes, set_send_gain, set_send_muted, set_send_pan,
            unassign_mix,
        },
        presets::{apply_preset, list_presets},
        system::get_system_info,
        telemetry::get_telemetry,
    },
    security::validate_origin,
    state::AppState,
    ws::ws_handler,
};
use axum::{
    extract::{connect_info::ConnectInfo, DefaultBodyLimit},
    middleware,
    routing::{get, post, put},
    Router,
};
use axum_test::{TestServer, WsMessage};
use bytes::Bytes;
use control_protocol::Role;
use control_server::ControlState;
use mix_engine::{Channel, Mix};
use serde_json::{json, Value};

// ── Ephemeral test-only Ed25519 PEM pair ────────────────────────────────────
// Generate keys at runtime so no private key is stored in the repository.
struct TestKeyDir(std::path::PathBuf);

#[cfg(unix)]
fn restrict_test_key_dir(path: &std::path::Path) {
    std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o700))
        .expect("test key directory permissions must be restricted");
}

#[cfg(not(unix))]
fn restrict_test_key_dir(_path: &std::path::Path) {}

impl Drop for TestKeyDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn test_keys() -> (Vec<u8>, Vec<u8>) {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be valid")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "open-iem-test-{}-{nonce}-{}",
        std::process::id(),
        rand::random::<u128>()
    ));
    fs::create_dir(&dir).expect("test key directory must be created exclusively");
    restrict_test_key_dir(&dir);
    let _cleanup = TestKeyDir(dir.clone());
    let private_path = dir.join("private.pem");
    let public_path = dir.join("public.pem");

    let status = std::process::Command::new("openssl")
        .args(["genpkey", "-algorithm", "ed25519", "-out"])
        .arg(&private_path)
        .status()
        .expect("openssl must be installed for integration tests");
    assert!(status.success(), "openssl key generation failed");
    let status = std::process::Command::new("openssl")
        .args(["pkey", "-in"])
        .arg(&private_path)
        .args(["-pubout", "-out"])
        .arg(&public_path)
        .status()
        .expect("openssl public-key export failed");
    assert!(status.success(), "openssl public-key export failed");

    let private = fs::read(&private_path).expect("generated private key must be readable");
    let public = fs::read(&public_path).expect("generated public key must be readable");
    (private, public)
}

const VALID_AUDIO_OFFER: &str = "v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\na=sendrecv\r\na=rtcp-mux\r\na=ice-ufrag:test\r\na=ice-pwd:testpassword\r\na=fingerprint:sha-256 00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00\r\na=setup:actpass\r\na=rtpmap:111 opus/48000/2\r\n";

const VALID_ICE_CANDIDATE: &str =
    "candidate:1 1 udp 2113937151 192.168.1.100 49152 typ host generation 0";

// ── Shared test fixture ────────────────────────────────────────────────────

fn build_test_app() -> (TestServer, AppState) {
    let (private_pem, public_pem) = test_keys();
    let jwt = JwtKeys::from_ed_pem(&private_pem, &public_pem).expect("test PEM must be valid");
    let db = Db::open_in_memory().expect("in-memory DB must open");
    let state = AppState::new(ControlState::new(), db, jwt);
    {
        let mut control = state.control.lock().expect("control lock");
        control
            .set_mix(0, Mix::new(0, "Mix 1"))
            .expect("test mix must configure");
        control
            .set_mix(1, Mix::new(1, "Mix 2"))
            .expect("test mix must configure");
    }

    let protected = Router::new()
        .route("/api/v1/state", get(get_state))
        .route("/api/v1/channels", get(list_channels))
        .route("/api/v1/telemetry", get(get_telemetry))
        .route("/api/v1/metrics", get(get_metrics))
        .route("/api/v1/presets", get(list_presets))
        .route("/api/v1/presets/{id}/apply", post(apply_preset))
        .route("/api/v1/audio/offer", post(offer))
        .route("/api/v1/audio/ice-candidate", post(ice_candidate))
        .route("/api/v1/audio/sessions", get(sessions))
        .route("/api/v1/audio/pairing", post(pair_device))
        .route(
            "/api/v1/audio/pairing/{device_id}",
            axum::routing::delete(revoke_device),
        )
        .route("/api/v1/channels/{index}/gain", put(set_channel_gain))
        .route("/api/v1/channels/{index}/mute", put(set_channel_mute))
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
        .route("/api/v1/auth/logout", post(logout))
        .route(
            "/api/v1/admin/users",
            get(admin_list_users).post(create_user),
        )
        .route(
            "/api/v1/admin/users/{id}",
            axum::routing::delete(admin_delete_user),
        )
        .route("/api/v1/admin/sessions", get(admin_list_sessions))
        .route(
            "/api/v1/admin/sessions/{id}",
            axum::routing::delete(admin_revoke_session),
        )
        .route("/ws/v1", get(ws_handler))
        .layer(middleware::from_fn_with_state(state.clone(), jwt_auth));

    let public = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/system", get(get_system_info))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/refresh", post(refresh));

    let app = Router::new()
        .merge(protected)
        .merge(public)
        .with_state(state.clone())
        .layer(axum::Extension(ConnectInfo(std::net::SocketAddr::from((
            [127, 0, 0, 1],
            8080,
        )))))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(middleware::from_fn(validate_origin));

    let server = TestServer::new(app);
    (server, state)
}

/// Seed a user into the in-memory DB and return a valid Bearer token.
fn seed_user_and_login(state: &AppState, username: &str, password: &str, role: Role) -> String {
    let pw_hash = hash_password(password).expect("hash must succeed");
    state
        .db
        .create_user(username, &pw_hash, role)
        .expect("create user must succeed");
    let (user_id, _, _, _) = state.db.find_user(username).expect("user must exist");
    // Issue token directly, while preserving persistent session association.
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
        .expect("session must be created");
    state
        .jwt
        .issue_with_session(username, user_id, role, &jti, Some(session_id))
        .expect("token must be issued")
}

// ── /api/v1/health ────────────────────────────────────────────────────────

#[tokio::test]
async fn health_returns_ok() {
    let (server, _state) = build_test_app();
    let resp = server.get("/api/v1/health").await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["status"], "ok");
}

// ── /api/v1/auth/login ────────────────────────────────────────────────────

#[tokio::test]
async fn login_with_valid_credentials_returns_200() {
    let (server, state) = build_test_app();
    let pw_hash = hash_password("pass123").expect("hash");
    state
        .db
        .create_user("alice", &pw_hash, Role::Musician)
        .expect("create user");

    let resp = server
        .post("/api/v1/auth/login")
        .add_header("Origin", "http://localhost")
        .json(&json!({"username": "alice", "password": "pass123"}))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert!(body["access_token"].is_string());
}

#[tokio::test]
async fn refresh_replay_revokes_replacement_access_token() {
    let (server, state) = build_test_app();
    let pw_hash = hash_password("pass123").expect("hash");
    state
        .db
        .create_user("refresh_replay", &pw_hash, Role::Musician)
        .expect("create user");

    let login_response = server
        .post("/api/v1/auth/login")
        .add_header("Origin", "http://localhost")
        .json(&json!({"username": "refresh_replay", "password": "pass123"}))
        .await;
    login_response.assert_status_ok();
    let cookie = login_response
        .headers()
        .get("set-cookie")
        .expect("login must set refresh cookie")
        .to_str()
        .expect("cookie must be valid header")
        .split(';')
        .next()
        .expect("cookie pair must exist")
        .to_owned();

    let refresh_response = server
        .post("/api/v1/auth/refresh")
        .add_header("Origin", "http://localhost")
        .add_header("Cookie", cookie.clone())
        .await;
    refresh_response.assert_status_ok();
    let replacement_access: Value = refresh_response.json();
    let replacement_access = replacement_access["access_token"]
        .as_str()
        .expect("refresh must return access token")
        .to_owned();

    let replay_response = server
        .post("/api/v1/auth/refresh")
        .add_header("Origin", "http://localhost")
        .add_header("Cookie", cookie)
        .await;
    replay_response.assert_status(axum::http::StatusCode::UNAUTHORIZED);

    server
        .get("/api/v1/state")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(replacement_access)
        .await
        .assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_with_wrong_password_returns_401() {
    let (server, state) = build_test_app();
    let pw_hash = hash_password("correct").expect("hash");
    state
        .db
        .create_user("bob", &pw_hash, Role::Musician)
        .expect("create user");

    let resp = server
        .post("/api/v1/auth/login")
        .add_header("Origin", "http://localhost")
        .json(&json!({"username": "bob", "password": "wrong"}))
        .await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_unknown_user_returns_401() {
    let (server, _state) = build_test_app();
    let resp = server
        .post("/api/v1/auth/login")
        .add_header("Origin", "http://localhost")
        .json(&json!({"username": "nobody", "password": "x"}))
        .await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ── Protected routes without token ─────────────────────────────────────────

#[tokio::test]
async fn state_requires_auth() {
    let (server, _state) = build_test_app();
    let resp = server.get("/api/v1/state").await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn audio_offer_requires_auth() {
    let (server, _state) = build_test_app();
    let resp = server
        .post("/api/v1/audio/offer")
        .add_header("Origin", "http://localhost")
        .json(&json!({"sdp": "v=0"}))
        .await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn audio_sessions_requires_auth() {
    let (server, _state) = build_test_app();
    let resp = server.get("/api/v1/audio/sessions").await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn musician_negotiates_offer_and_trickles_ice_candidate_over_http() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "audio_musician", "pw", Role::Musician);

    let offer_response = server
        .post("/api/v1/audio/offer")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token.clone())
        .json(&json!({"sdp": VALID_AUDIO_OFFER, "mix_id": null}))
        .await;
    offer_response.assert_status_ok();
    let offer_body: Value = offer_response.json();
    assert!(offer_body["sdp"]
        .as_str()
        .is_some_and(|sdp| sdp.starts_with("v=0")));

    let candidate_response = server
        .post("/api/v1/audio/ice-candidate")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token)
        .json(&json!({"candidate": VALID_ICE_CANDIDATE}))
        .await;
    candidate_response.assert_status_ok();
    let candidate_body: Value = candidate_response.json();
    assert_eq!(candidate_body, json!({"accepted": true}));
}

// ── Role enforcement ────────────────────────────────────────────────────────

#[tokio::test]
async fn musician_cannot_list_audio_sessions() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "mus1", "pw", Role::Musician);
    let resp = server
        .get("/api/v1/audio/sessions")
        .authorization_bearer(token)
        .await;
    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn engineer_can_list_audio_sessions() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "eng1", "pw", Role::Engineer);
    let resp = server
        .get("/api/v1/audio/sessions")
        .authorization_bearer(token)
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert!(body["sessions"].is_array());
}

#[tokio::test]
async fn musician_can_access_state() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "mus2", "pw", Role::Musician);
    let resp = server
        .get("/api/v1/state")
        .authorization_bearer(token)
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["schema_version"], 1);
    assert_eq!(body["revision"], 2);
    assert_eq!(body["channels"].as_array().map(Vec::len), Some(0));
    assert_eq!(body["mixes"].as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn telemetry_requires_engineer_and_reports_simulated_backend() {
    let (server, state) = build_test_app();
    let musician = seed_user_and_login(&state, "telemetry_mus", "pw", Role::Musician);
    server
        .get("/api/v1/telemetry")
        .authorization_bearer(musician)
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);

    let engineer = seed_user_and_login(&state, "telemetry_eng", "pw", Role::Engineer);
    let response = server
        .get("/api/v1/telemetry")
        .authorization_bearer(engineer)
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["schema_version"], 1);
    assert_eq!(body["availability"], "simulated");
    assert_eq!(body["backend"], "simulated");
    assert!(body["sample_rate_hz"].is_null());
    assert!(body["frames_processed"].is_null());
    assert!(body["xrun_count"].is_null());
}

#[tokio::test]
async fn musician_cannot_create_user() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "mus3", "pw", Role::Musician);
    let resp = server
        .post("/api/v1/admin/users")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token)
        .json(&json!({"username": "x", "password": "pw2", "role": "MUSICIAN"}))
        .await;
    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_can_create_user() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "adm1", "pw", Role::Admin);
    let resp = server
        .post("/api/v1/admin/users")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token)
        .json(&json!({"username": "newuser", "password": "pw2", "role": "MUSICIAN"}))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
}

// ── Mix assignment and musician ownership ───────────────────────────────────

#[tokio::test]
async fn engineer_assigns_mix_and_musician_controls_owned_send() {
    let (server, state) = build_test_app();
    let engineer = seed_user_and_login(&state, "eng_mix", "pw", Role::Engineer);
    let musician = seed_user_and_login(&state, "mus_mix", "pw", Role::Musician);
    let (musician_id, _, _, _) = state.db.find_user("mus_mix").unwrap();

    let assigned = server
        .post("/api/v1/mixes/0/assign")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&engineer)
        .json(&json!({"user_id": musician_id}))
        .await;
    assigned.assert_status_ok();

    let updated = server
        .put("/api/v1/mixes/0/sends/2/gain")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&musician)
        .json(&json!({"gain_db": -12.0}))
        .await;
    updated.assert_status_ok();
    let body: Value = updated.json();
    assert_eq!(body["gain_db"], -12.0);
}

#[tokio::test]
async fn musician_cannot_control_unassigned_mix() {
    let (server, state) = build_test_app();
    let musician = seed_user_and_login(&state, "mus_other", "pw", Role::Musician);
    let response = server
        .get("/api/v1/mixes/1/sends/0")
        .authorization_bearer(&musician)
        .await;
    response.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn musician_send_rejects_invalid_gain() {
    let (server, state) = build_test_app();
    let engineer = seed_user_and_login(&state, "eng_invalid", "pw", Role::Engineer);
    let musician = seed_user_and_login(&state, "mus_invalid", "pw", Role::Musician);
    let (id, _, _, _) = state.db.find_user("mus_invalid").unwrap();
    server
        .post("/api/v1/mixes/0/assign")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&engineer)
        .json(&json!({"user_id": id}))
        .await
        .assert_status_ok();
    let response = server
        .put("/api/v1/mixes/0/sends/0/gain")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&musician)
        .json(&json!({"gain_db": 999.0}))
        .await;
    response.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

// ── CSRF / Origin checks ────────────────────────────────────────────────────

#[tokio::test]
async fn post_with_unknown_origin_is_rejected() {
    let (server, _state) = build_test_app();
    let resp = server
        .post("/api/v1/auth/login")
        .add_header("Origin", "https://evil.example")
        .json(&json!({"username": "x", "password": "y"}))
        .await;
    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_without_origin_is_allowed() {
    // GET /health has no Origin requirement and no state mutation.
    let (server, _state) = build_test_app();
    let resp = server.get("/api/v1/health").await;
    resp.assert_status_ok();
}

// ── /api/v1/audio/offer — mix ownership ─────────────────────────────────────

#[tokio::test]
async fn musician_cannot_offer_unassigned_mix() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "mus7", "pw", Role::Musician);
    let resp = server
        .post("/api/v1/audio/offer")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token)
        .json(&json!({"sdp": "invalid-but-never-parsed", "mix_id": "0"}))
        .await;
    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}

// ── /api/v1/audio/offer — bad SDP payload ──────────────────────────────────

#[tokio::test]
async fn offer_with_bad_sdp_returns_400() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "mus4", "pw", Role::Musician);
    let resp = server
        .post("/api/v1/audio/offer")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token)
        .json(&json!({"sdp": "this-is-not-valid-sdp"}))
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

// ── /api/v1/audio/ice-candidate ─────────────────────────────────────────────

#[tokio::test]
async fn ice_candidate_without_session_returns_400() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "mus5", "pw", Role::Musician);
    // No offer was made first, so no session exists.
    let resp = server
        .post("/api/v1/audio/ice-candidate")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token)
        .json(&json!({"candidate": "candidate:1 1 udp 2113937151 192.168.1.1 49152 typ host generation 0"}))
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn ice_candidate_with_malformed_string_returns_400() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "mus6", "pw", Role::Musician);
    let resp = server
        .post("/api/v1/audio/ice-candidate")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token)
        .json(&json!({"candidate": "not-a-candidate"}))
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn engineer_applies_builtin_preset_to_channel() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "preset_eng", "pw", Role::Engineer);
    let response = server
        .post("/api/v1/presets/default-vocal/apply")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token)
        .json(&json!({"channel_index": 2}))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["preset_id"], "default-vocal");
    assert_eq!(body["channel_index"], 2);
    assert_eq!(body["applied"], true);
    let ctrl = state.control.lock().unwrap();
    assert!(ctrl.channel(2).unwrap().gain_db().abs() < f32::EPSILON);
    assert!(!ctrl.channel(2).unwrap().muted);
}

#[tokio::test]
async fn admin_applies_builtin_preset_to_channel() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "preset_admin", "pw", Role::Admin);
    let response = server
        .post("/api/v1/presets/default-vocal/apply")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token)
        .json(&json!({"channel_index": 2}))
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["preset_id"], "default-vocal");
    assert_eq!(body["channel_index"], 2);
    assert_eq!(body["applied"], true);
    assert!(body["revision"].as_u64().is_some());
    let ctrl = state.control.lock().unwrap();
    assert!(ctrl.channel(2).unwrap().gain_db().abs() < f32::EPSILON);
    assert!(!ctrl.channel(2).unwrap().muted);
}

#[tokio::test]
async fn engineer_rejects_out_of_range_preset_channel_without_mutation() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "preset_eng_range", "pw", Role::Engineer);
    let revision_before = state.control.lock().unwrap().revision();
    server
        .post("/api/v1/presets/default-vocal/apply")
        .authorization_bearer(token)
        .json(&json!({"channel_index": 8}))
        .await
        .assert_status(axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(state.control.lock().unwrap().revision(), revision_before);
    assert!(state.control.lock().unwrap().channel(8).is_none());
}

#[tokio::test]
async fn engineer_rejects_preset_on_locked_channel_without_mutation() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "preset_eng_locked", "pw", Role::Engineer);
    let mut locked = control_server::ControlState::new();
    let mut channel = mix_engine::Channel::new(2, "Locked");
    channel.set_gain_db(7.0);
    channel.set_muted(true);
    channel.set_locked(true);
    locked.set_channel(2, channel).unwrap();
    *state.control.lock().unwrap() = locked;
    let revision_before = state.control.lock().unwrap().revision();

    server
        .post("/api/v1/presets/default-vocal/apply")
        .authorization_bearer(token)
        .json(&json!({"channel_index": 2}))
        .await
        .assert_status(axum::http::StatusCode::BAD_REQUEST);

    let ctrl = state.control.lock().unwrap();
    assert_eq!(ctrl.revision(), revision_before);
    let channel = ctrl.channel(2).unwrap();
    assert!(channel.locked);
    assert!((channel.gain_db() - 7.0).abs() < f32::EPSILON);
    assert!(channel.muted);
}

#[tokio::test]
async fn engineer_rejects_unknown_preset_fields_without_mutation() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "preset_eng_payload", "pw", Role::Engineer);
    let revision_before = state.control.lock().unwrap().revision();
    server
        .post("/api/v1/presets/default-vocal/apply")
        .authorization_bearer(token)
        .json(&json!({"channel_index": 2, "gain_db": 12.0}))
        .await
        .assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(state.control.lock().unwrap().revision(), revision_before);
    assert!(state.control.lock().unwrap().channel(2).is_none());
}

#[tokio::test]
async fn musician_cannot_apply_preset_and_invalid_input_does_not_mutate() {
    let (server, state) = build_test_app();
    let musician = seed_user_and_login(&state, "preset_mus", "pw", Role::Musician);
    server
        .post("/api/v1/presets/default-vocal/apply")
        .authorization_bearer(musician)
        .json(&json!({"channel_index": 2}))
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);
    let engineer = seed_user_and_login(&state, "preset_eng_bad", "pw", Role::Engineer);
    server
        .post("/api/v1/presets/unknown/apply")
        .authorization_bearer(engineer)
        .json(&json!({"channel_index": 2}))
        .await
        .assert_status(axum::http::StatusCode::BAD_REQUEST);
    assert!(state.control.lock().unwrap().channel(2).is_none());
}

// ── /api/v1/channels ────────────────────────────────────────────────────────

#[tokio::test]
async fn set_gain_out_of_range_returns_400() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "eng2", "pw", Role::Engineer);
    let resp = server
        .put("/api/v1/channels/0/gain")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token)
        .json(&json!({"gain_db": 999.0}))
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn set_mute_with_valid_payload_returns_200() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "eng3", "pw", Role::Engineer);
    let resp = server
        .put("/api/v1/channels/0/mute")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(token)
        .json(&json!({"muted": true}))
        .await;
    resp.assert_status_ok();
}

// ── Admin: user listing ──────────────────────────────────────────────────────

#[tokio::test]
async fn admin_list_users_returns_all_users() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "admin_lu1", "pw", Role::Admin);
    let pw_hash = hash_password("pw2").unwrap();
    state
        .db
        .create_user("musician_lu1", &pw_hash, Role::Musician)
        .unwrap();
    let resp = server
        .get("/api/v1/admin/users")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&token)
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    let arr = body.as_array().expect("should be array");
    assert!(arr.len() >= 2);
    let usernames: Vec<&str> = arr.iter().filter_map(|u| u["username"].as_str()).collect();
    assert!(usernames.contains(&"admin_lu1"));
    assert!(usernames.contains(&"musician_lu1"));
}

#[tokio::test]
async fn admin_list_users_rejected_for_non_admin() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "eng_lu1", "pw", Role::Engineer);
    let resp = server
        .get("/api/v1/admin/users")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&token)
        .await;
    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}

// ── Admin: user deletion ─────────────────────────────────────────────────────

#[tokio::test]
async fn admin_delete_user_returns_204() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "admin_du1", "pw", Role::Admin);
    let pw_hash = hash_password("pw").unwrap();
    state
        .db
        .create_user("todelete1", &pw_hash, Role::Musician)
        .unwrap();
    let (user_id, _, _, _) = state.db.find_user("todelete1").unwrap();
    let resp = server
        .delete(&format!("/api/v1/admin/users/{user_id}"))
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&token)
        .await;
    resp.assert_status(axum::http::StatusCode::NO_CONTENT);
    assert!(state.db.find_user("todelete1").is_err());
}

#[tokio::test]
async fn admin_delete_nonexistent_user_returns_404() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "admin_du2", "pw", Role::Admin);
    let resp = server
        .delete("/api/v1/admin/users/99999")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&token)
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

// ── Admin: self-delete protection ───────────────────────────────────────────

#[tokio::test]
async fn admin_cannot_delete_own_account() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "admin_self", "pw", Role::Admin);
    let (caller_id, _, _, _) = state.db.find_user("admin_self").unwrap();
    let resp = server
        .delete(&format!("/api/v1/admin/users/{caller_id}"))
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&token)
        .await;
    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
    let body: Value = resp.json();
    assert_eq!(body["code"], "FORBIDDEN");
}

#[tokio::test]
async fn admin_list_sessions_returns_active_sessions() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "admin_sl1", "pw", Role::Admin);
    let resp = server
        .get("/api/v1/admin/sessions")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&token)
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert!(body.is_array());
}

#[tokio::test]
async fn admin_list_sessions_rejected_for_musician() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "mus_sl1", "pw", Role::Musician);
    let resp = server
        .get("/api/v1/admin/sessions")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&token)
        .await;
    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}

// ── Admin: session revocation ────────────────────────────────────────────────

#[tokio::test]
async fn admin_revoke_nonexistent_session_returns_404() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "admin_sr1", "pw", Role::Admin);
    let resp = server
        .delete("/api/v1/admin/sessions/99999")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&token)
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn admin_revoke_session_by_id_returns_204() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "admin_sr2", "pw", Role::Admin);
    // Directly store a refresh token so we have a known session ID to revoke.
    let (user_id, _, _, _) = state.db.find_user("admin_sr2").unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    state
        .db
        .store_refresh_token(user_id, "test_tok_hash_sr2", now + 3600, "fam-sr2")
        .unwrap();
    // List sessions to find the ID we just stored.
    let sessions = state.db.list_active_sessions(now).unwrap();
    let session_id = sessions
        .iter()
        .find(|(_, uid, _)| *uid == user_id)
        .map(|(id, _, _)| *id)
        .expect("session must exist");
    let resp = server
        .delete(&format!("/api/v1/admin/sessions/{session_id}"))
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&token)
        .await;
    resp.assert_status(axum::http::StatusCode::NO_CONTENT);
    // Verify session is now revoked (not in active list).
    let after = state.db.list_active_sessions(now).unwrap();
    assert!(!after.iter().any(|(id, _, _)| *id == session_id));
}

// ── Phase 31: post-issuance revocation ───────────────────────────────────────

#[tokio::test]
async fn refresh_rotation_invalidates_old_access_mapping() {
    let (_server, state) = build_test_app();
    let token = seed_user_and_login(&state, "phase31_rotation", "pw", Role::Engineer);
    let old_claims = state.jwt.verify(&token).expect("old token must verify");
    let old_refresh = generate_refresh_token();
    let old_hash = token_to_storage_key(&old_refresh);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_secs();
    let (user_id, _, _, _) = state.db.find_user("phase31_rotation").unwrap();
    let old_session_id = state
        .db
        .create_session_with_access(
            user_id,
            &old_hash,
            now + 3600,
            "phase31-family",
            "phase31-old-jti",
            now + 3600,
        )
        .unwrap();
    assert!(state
        .db
        .is_access_session_active("phase31-old-jti", user_id, old_session_id, now)
        .unwrap());

    let new_hash = token_to_storage_key(&generate_refresh_token());
    let (rotated_user_id, _, new_session_id) = state
        .db
        .rotate_refresh_token_with_access_id(
            &old_hash,
            &new_hash,
            now + 3600,
            now,
            Some("phase31-new-jti"),
            Some(now + 3600),
        )
        .unwrap();

    assert_eq!(rotated_user_id, user_id);
    assert_eq!(old_claims.user_id, user_id);
    assert!(!state
        .db
        .is_access_session_active("phase31-old-jti", user_id, old_session_id, now)
        .unwrap());
    assert!(state
        .db
        .is_access_session_active("phase31-new-jti", user_id, new_session_id, now)
        .unwrap());
}

#[tokio::test]
async fn session_revoke_invalidates_access_middleware_mapping() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "phase31_session", "pw", Role::Engineer);
    let claims = state.jwt.verify(&token).unwrap();
    state
        .db
        .revoke_session_by_id(claims.session_id.unwrap())
        .unwrap();

    let response = server
        .get("/api/v1/state")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&token)
        .await;
    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn user_deletion_invalidates_access_middleware_mapping() {
    let (server, state) = build_test_app();
    let token = seed_user_and_login(&state, "phase31_deleted", "pw", Role::Engineer);
    let claims = state.jwt.verify(&token).unwrap();
    state.db.delete_user_with_sessions(claims.user_id).unwrap();

    let response = server
        .get("/api/v1/state")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&token)
        .await;
    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn established_websocket_rejects_message_after_session_revocation() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "phase31_ws", "pw", Role::Engineer);
    let claims = state.jwt.verify(&token).unwrap();
    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    state
        .db
        .revoke_session_by_id(claims.session_id.unwrap())
        .unwrap();
    ws.send_message(WsMessage::Text(ws_envelope("GetState", json!({})).into()))
        .await;
    let response: Value = ws.receive_json().await;
    assert_eq!(response["payload"]["type"], "Error");
    assert_eq!(response["payload"]["data"]["code"], "SESSION_REVOKED");
}

// ── WebSocket: send mutations ────────────────────────────────────────────────
//
// These tests exercise the ws_handler through axum-test's HTTP transport,
// which supports WebSocket upgrades.  Each test creates a fresh app state so
// mix configuration and DB state are isolated.

fn build_ws_app() -> (axum_test::TestServer, AppState) {
    let (private_pem, public_pem) = test_keys();
    let jwt = JwtKeys::from_ed_pem(&private_pem, &public_pem).expect("test PEM must be valid");
    let db = Db::open_in_memory().expect("in-memory DB must open");
    let state = AppState::new(ControlState::new(), db, jwt);
    {
        let mut control = state.control.lock().expect("control lock");
        control
            .set_mix(0, Mix::new(0, "Mix WS 1"))
            .expect("set mix 0");
        control
            .set_mix(1, Mix::new(1, "Mix WS 2"))
            .expect("set mix 1");
    }

    let protected = Router::new()
        .route("/api/v1/state", get(get_state))
        .route("/api/v1/channels", get(list_channels))
        .route("/api/v1/telemetry", get(get_telemetry))
        .route("/api/v1/audio/offer", post(offer))
        .route("/api/v1/audio/ice-candidate", post(ice_candidate))
        .route("/api/v1/audio/sessions", get(sessions))
        .route("/api/v1/audio/pairing", post(pair_device))
        .route(
            "/api/v1/audio/pairing/{device_id}",
            axum::routing::delete(revoke_device),
        )
        .route("/api/v1/channels/{index}/gain", put(set_channel_gain))
        .route("/api/v1/channels/{index}/mute", put(set_channel_mute))
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
        .route("/api/v1/auth/logout", post(logout))
        .route(
            "/api/v1/admin/users",
            get(admin_list_users).post(create_user),
        )
        .route(
            "/api/v1/admin/users/{id}",
            axum::routing::delete(admin_delete_user),
        )
        .route("/api/v1/admin/sessions", get(admin_list_sessions))
        .route(
            "/api/v1/admin/sessions/{id}",
            axum::routing::delete(admin_revoke_session),
        )
        .route("/ws/v1", get(ws_handler))
        .layer(middleware::from_fn_with_state(state.clone(), jwt_auth));

    let public = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/system", get(get_system_info))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/refresh", post(refresh));

    let app = Router::new()
        .merge(protected)
        .merge(public)
        .with_state(state.clone())
        .layer(axum::Extension(ConnectInfo(std::net::SocketAddr::from((
            [127, 0, 0, 1],
            8080,
        )))))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(middleware::from_fn(validate_origin));

    // WebSocket tests require the HTTP transport (not mock).
    let server = axum_test::TestServer::builder().http_transport().build(app);

    (server, state)
}

#[allow(clippy::needless_pass_by_value)]
fn ws_envelope(msg_type: &str, data: Value) -> String {
    serde_json::to_string(&json!({
        "version": 1,
        "request_id": "test-req-1",
        "payload": {
            "type": msg_type,
            "data": data
        }
    }))
    .unwrap()
}

#[tokio::test]
async fn ws_binary_frame_returns_protocol_error_and_closes() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "eng_ws_binary", "pw", Role::Engineer);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_message(WsMessage::Binary(Bytes::from_static(b"invalid")))
        .await;
    let response: Value = ws.receive_json().await;
    assert_eq!(response["payload"]["type"], "Error");
    assert_eq!(response["payload"]["data"]["code"], "INVALID_MESSAGE");
}

#[tokio::test]
async fn ws_oversized_text_message_is_rejected_by_upgrade_limit() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "eng_ws_oversized", "pw", Role::Engineer);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    let oversized = "x".repeat(16 * 1024 + 1);
    ws.send_message(WsMessage::Text(oversized.into())).await;

    let receive = tokio::spawn(async move { ws.receive_message().await });
    let panic = receive
        .await
        .expect_err("oversized message must terminate WebSocket transport");
    assert!(panic.is_panic());
    let panic_message = panic.into_panic();
    let panic_message = panic_message
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic_message.downcast_ref::<&str>().copied())
        .unwrap_or_default();
    assert!(
        panic_message.contains("Connection reset without closing handshake"),
        "unexpected WebSocket rejection: {panic_message}"
    );
}

#[tokio::test]
async fn ws_client_ping_receives_matching_pong() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "eng_ws_ping", "pw", Role::Engineer);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    let payload = Bytes::from_static(b"keepalive-test");
    ws.send_message(WsMessage::Ping(payload.clone())).await;
    match ws.receive_message().await {
        WsMessage::Pong(received) => assert_eq!(received, payload),
        other => panic!("expected Pong, got {other:?}"),
    }
}

#[tokio::test]
async fn ws_engineer_set_send_gain_returns_send_ack() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "eng_ws1", "pw", Role::Engineer);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetSendGain",
        json!({"mix_index": 0, "channel_index": 0, "gain_db": -6.0}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "SendAck");
    let data = &resp["payload"]["data"];
    assert_eq!(data["mix_index"], 0);
    assert_eq!(data["channel_index"], 0);
    let gain: f64 = data["gain_db"].as_f64().unwrap();
    assert!((gain - (-6.0)).abs() < 0.001);
}

#[tokio::test]
async fn ws_engineer_set_send_pan_returns_send_ack() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "eng_ws2", "pw", Role::Engineer);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetSendPan",
        json!({"mix_index": 0, "channel_index": 1, "pan": 0.5}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "SendAck");
    let pan: f64 = resp["payload"]["data"]["pan"].as_f64().unwrap();
    assert!((pan - 0.5).abs() < 0.001);
}

#[tokio::test]
async fn ws_engineer_set_send_muted_returns_send_ack() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "eng_ws3", "pw", Role::Engineer);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetSendMuted",
        json!({"mix_index": 0, "channel_index": 2, "muted": true}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "SendAck");
    assert_eq!(resp["payload"]["data"]["muted"], true);
}

#[tokio::test]
async fn ws_musician_db_failure_denies_set_send_gain() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "mus_ws_db_failure", "pw", Role::Musician);
    // Drop ownership table after authentication. Lookup now returns DB error;
    // authorization must fail closed rather than allowing the mutation.
    state.db.drop_mix_assignments_table_for_test().unwrap();

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetSendGain",
        json!({"mix_index": 0, "channel_index": 0, "gain_db": 0.0}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "Error");
    assert_eq!(resp["payload"]["data"]["code"], "FORBIDDEN");
}

#[tokio::test]
async fn ws_musician_denied_set_send_gain_on_unassigned_mix() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "mus_ws1", "pw", Role::Musician);
    // Musician has NO mix assignment — any send mutation must be rejected.

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetSendGain",
        json!({"mix_index": 0, "channel_index": 0, "gain_db": 0.0}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "Error");
    assert_eq!(resp["payload"]["data"]["code"], "FORBIDDEN");
    assert_eq!(resp["request_id"], "test-req-1");
}

#[tokio::test]
async fn ws_musician_allowed_set_send_gain_on_assigned_mix() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "mus_ws2", "pw", Role::Musician);
    // Assign mix 1 to the musician.
    let (user_id, _, _, _) = state.db.find_user("mus_ws2").unwrap();
    state.db.assign_mix(1, user_id).unwrap();

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetSendGain",
        json!({"mix_index": 1, "channel_index": 0, "gain_db": -3.0}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "SendAck");
    let gain: f64 = resp["payload"]["data"]["gain_db"].as_f64().unwrap();
    assert!((gain - (-3.0)).abs() < 0.001);
}

#[tokio::test]
async fn ws_musician_denied_channel_gain_mutation() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "mus_ws3", "pw", Role::Musician);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    // Musician cannot call SetChannelGain
    let msg = serde_json::to_string(&json!({
        "version": 1,
        "request_id": "test-req-ch",
        "payload": {
            "type": "SetChannelGain",
            "data": {"channel": 0, "gain_db": 0.0}
        }
    }))
    .unwrap();
    ws.send_text(msg).await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "Error");
    assert_eq!(resp["payload"]["data"]["code"], "FORBIDDEN");
}

#[tokio::test]
async fn ws_invalid_gain_nan_returns_error() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "eng_ws4", "pw", Role::Engineer);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    // Send a non-finite gain_db (NaN cannot serialize, use 999.0 out-of-range)
    ws.send_text(ws_envelope(
        "SetSendGain",
        json!({"mix_index": 0, "channel_index": 0, "gain_db": 200.0}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "Error");
    assert_eq!(resp["payload"]["data"]["code"], "INVALID_GAIN");
}

// ── WebSocket broadcast: delta received by observer ────────────────────────
//
// Two clients connect simultaneously (engineer mutator + engineer observer).
// Mutator sends SetSendGain; observer must receive an unsolicited SendAck
// broadcast within a short timeout.

#[tokio::test]
async fn ws_send_mutation_broadcasts_to_other_sessions() {
    let (server, state) = build_ws_app();
    let mutator_token = seed_user_and_login(&state, "eng_broadcast_mut", "pw", Role::Engineer);
    let observer_token = seed_user_and_login(&state, "eng_broadcast_obs", "pw", Role::Engineer);

    let mut mutator = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{mutator_token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    let mut observer = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{observer_token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    // Prove observer handler reached its receive loop. Subscription happens
    // before that loop, so this removes handshake/task-scheduling ambiguity.
    observer
        .send_text(ws_envelope(
            "SetSendGain",
            json!({"mix_index": 0, "channel_index": 1, "gain_db": 999.0}),
        ))
        .await;
    let observer_ready: Value = observer.receive_json().await;
    assert_eq!(observer_ready["payload"]["type"], "Error");

    // Mutator changes gain on mix 0, channel 1.
    mutator
        .send_text(ws_envelope(
            "SetSendGain",
            json!({"mix_index": 0, "channel_index": 1, "gain_db": -9.0}),
        ))
        .await;

    // Mutator receives its own SendAck.
    let mutator_resp: Value = mutator.receive_json().await;
    assert_eq!(mutator_resp["payload"]["type"], "SendAck");

    // Observer must receive an unsolicited broadcast SendAck.
    let obs_resp: Value =
        tokio::time::timeout(std::time::Duration::from_secs(5), observer.receive_json())
            .await
            .expect("observer did not receive broadcast within 5 s");

    assert_eq!(obs_resp["payload"]["type"], "SendAck");
    assert_eq!(obs_resp["payload"]["data"]["mix_index"], 0);
    assert_eq!(obs_resp["payload"]["data"]["channel_index"], 1);
    let gain: f64 = obs_resp["payload"]["data"]["gain_db"].as_f64().unwrap();
    assert!((gain - (-9.0)).abs() < 0.001);
}

// ── Phase 23: WebSocket master gain / mute ──────────────────────────────────

#[tokio::test]
async fn ws_engineer_set_master_gain_returns_master_ack() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "eng_mg1", "pw", Role::Engineer);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetMasterGain",
        json!({"mix_index": 0, "gain_db": -6.0}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "MasterAck");
    let data = &resp["payload"]["data"];
    assert_eq!(data["mix_index"], 0);
    let gain: f64 = data["master_gain_db"].as_f64().unwrap();
    assert!((gain - (-6.0)).abs() < 0.001);
    assert_eq!(data["master_muted"], false);
}

#[tokio::test]
async fn ws_engineer_set_master_mute_returns_master_ack() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "eng_mm1", "pw", Role::Engineer);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetMasterMute",
        json!({"mix_index": 0, "muted": true}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "MasterAck");
    assert_eq!(resp["payload"]["data"]["master_muted"], true);
}

#[tokio::test]
async fn ws_musician_denied_set_master_gain() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "mus_mg1", "pw", Role::Musician);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetMasterGain",
        json!({"mix_index": 0, "gain_db": 0.0}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "Error");
    assert_eq!(resp["payload"]["data"]["code"], "FORBIDDEN");
}

#[tokio::test]
async fn ws_musician_denied_set_master_mute() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "mus_mm1", "pw", Role::Musician);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetMasterMute",
        json!({"mix_index": 0, "muted": true}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "Error");
    assert_eq!(resp["payload"]["data"]["code"], "FORBIDDEN");
}

#[tokio::test]
async fn ws_engineer_set_master_gain_invalid_gain_returns_error() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "eng_mg_inv", "pw", Role::Engineer);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetMasterGain",
        json!({"mix_index": 0, "gain_db": 999.0}),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "Error");
    assert_eq!(resp["payload"]["data"]["code"], "INVALID_GAIN");
}

#[tokio::test]
async fn ws_master_mutation_broadcasts_to_other_sessions() {
    let (server, state) = build_ws_app();
    let mutator_token = seed_user_and_login(&state, "eng_master_mut", "pw", Role::Engineer);
    let observer_token = seed_user_and_login(&state, "eng_master_obs", "pw", Role::Engineer);

    let mut mutator = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{mutator_token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    let mut observer = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{observer_token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    // Prove observer handler reached its receive loop before mutation.
    observer
        .send_text(ws_envelope(
            "SetMasterGain",
            json!({"mix_index": 0, "gain_db": 999.0}),
        ))
        .await;
    let observer_ready: Value = observer.receive_json().await;
    assert_eq!(observer_ready["payload"]["type"], "Error");

    mutator
        .send_text(ws_envelope(
            "SetMasterGain",
            json!({"mix_index": 0, "gain_db": -12.0}),
        ))
        .await;

    // Mutator receives its own MasterAck.
    let mutator_resp: Value = mutator.receive_json().await;
    assert_eq!(mutator_resp["payload"]["type"], "MasterAck");

    // Observer must receive an unsolicited broadcast MasterAck.
    let obs_resp: Value =
        tokio::time::timeout(std::time::Duration::from_secs(5), observer.receive_json())
            .await
            .expect("observer did not receive broadcast within 5 s");

    assert_eq!(obs_resp["payload"]["type"], "MasterAck");
    assert_eq!(obs_resp["payload"]["data"]["mix_index"], 0);
    let gain: f64 = obs_resp["payload"]["data"]["master_gain_db"]
        .as_f64()
        .unwrap();
    assert!((gain - (-12.0)).abs() < 0.001);
}

// ── WebSocket master broadcast: Musician receives only own mix ─────────────

#[tokio::test]
async fn ws_master_broadcast_filtered_by_musician_assignment() {
    let (server, state) = build_ws_app();
    let eng_token = seed_user_and_login(&state, "eng_mf_mut", "pw", Role::Engineer);
    let mus0_token = seed_user_and_login(&state, "mus_mf_0", "pw", Role::Musician);
    let mus1_token = seed_user_and_login(&state, "mus_mf_1", "pw", Role::Musician);

    let (mus0_id, _, _, _) = state.db.find_user("mus_mf_0").unwrap();
    let (mus1_id, _, _, _) = state.db.find_user("mus_mf_1").unwrap();
    state.db.assign_mix(0, mus0_id).unwrap();
    state.db.assign_mix(1, mus1_id).unwrap();

    let mut eng = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{eng_token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    let mut mus0 = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{mus0_token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    let mut mus1 = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{mus1_token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    eng.send_text(ws_envelope(
        "SetMasterGain",
        json!({"mix_index": 0, "gain_db": -6.0}),
    ))
    .await;

    let eng_resp: Value = eng.receive_json().await;
    assert_eq!(eng_resp["payload"]["type"], "MasterAck");

    // Musician on mix 0 must receive broadcast.
    let mus0_resp: Value =
        tokio::time::timeout(std::time::Duration::from_secs(5), mus0.receive_json())
            .await
            .expect("musician 0 must receive master broadcast for own mix");
    assert_eq!(mus0_resp["payload"]["type"], "MasterAck");
    assert_eq!(mus0_resp["payload"]["data"]["mix_index"], 0);

    // Musician on mix 1 must NOT receive broadcast for mix 0.
    let mus1_no_recv: Result<_, _> = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        mus1.receive_json::<Value>(),
    )
    .await;
    assert!(
        mus1_no_recv.is_err(),
        "musician assigned to mix 1 must NOT receive master broadcast for mix 0"
    );
}

// ── Phase 92: WebSocket EQ band control ────────────────────────────────────

#[tokio::test]
async fn ws_engineer_set_eq_band_returns_eq_band_ack() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "eng_eq1", "pw", Role::Engineer);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetEqBand",
        json!({
            "mix_index": 0,
            "band_index": 1,
            "frequency_hz": 1000.0,
            "gain_db": 6.0,
            "q": 1.4,
            "enabled": true
        }),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "EqBandAck");
    assert_eq!(resp["payload"]["data"]["mix_index"], 0);
    assert_eq!(resp["payload"]["data"]["band_index"], 1);
    assert!(resp["payload"]["data"]["enabled"].as_bool().unwrap());
    let freq: f64 = resp["payload"]["data"]["frequency_hz"].as_f64().unwrap();
    assert!((freq - 1000.0).abs() < 0.1);
    let gain: f64 = resp["payload"]["data"]["gain_db"].as_f64().unwrap();
    assert!((gain - 6.0).abs() < 0.001);
}

#[tokio::test]
async fn ws_musician_denied_set_eq_band() {
    let (server, state) = build_ws_app();
    let token = seed_user_and_login(&state, "mus_eq1", "pw", Role::Musician);

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    ws.send_text(ws_envelope(
        "SetEqBand",
        json!({
            "mix_index": 0,
            "band_index": 0,
            "frequency_hz": 1000.0,
            "gain_db": 0.0,
            "q": 1.0,
            "enabled": false
        }),
    ))
    .await;

    let resp: Value = ws.receive_json().await;
    assert_eq!(resp["payload"]["type"], "Error");
    assert_eq!(resp["payload"]["data"]["code"], "FORBIDDEN");
}

#[tokio::test]
async fn ws_eq_band_mutation_broadcasts_to_other_engineer_sessions() {
    let (server, state) = build_ws_app();
    let mutator_token = seed_user_and_login(&state, "eng_eq_mut", "pw", Role::Engineer);
    let observer_token = seed_user_and_login(&state, "eng_eq_obs", "pw", Role::Engineer);

    let mut mutator = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{mutator_token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    let mut observer = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{observer_token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    // Prime observer to ensure it has reached its receive loop.
    observer
        .send_text(ws_envelope(
            "SetEqBand",
            json!({
                "mix_index": 0,
                "band_index": 9,   // out of range → Error response
                "frequency_hz": 1000.0,
                "gain_db": 0.0,
                "q": 1.0,
                "enabled": false
            }),
        ))
        .await;
    let _: Value = observer.receive_json().await; // consume the Error

    mutator
        .send_text(ws_envelope(
            "SetEqBand",
            json!({
                "mix_index": 0,
                "band_index": 2,
                "frequency_hz": 2000.0,
                "gain_db": -3.0,
                "q": 0.7,
                "enabled": true
            }),
        ))
        .await;

    let mutator_resp: Value = mutator.receive_json().await;
    assert_eq!(mutator_resp["payload"]["type"], "EqBandAck");

    let obs_resp: Value =
        tokio::time::timeout(std::time::Duration::from_secs(5), observer.receive_json())
            .await
            .expect("observer did not receive EQ band broadcast within 5 s");

    assert_eq!(obs_resp["payload"]["type"], "EqBandAck");
    assert_eq!(obs_resp["payload"]["data"]["band_index"], 2);
    let gain: f64 = obs_resp["payload"]["data"]["gain_db"].as_f64().unwrap();
    assert!((gain - (-3.0)).abs() < 0.001);
}

#[tokio::test]
async fn ws_eq_band_broadcast_not_forwarded_to_musician() {
    let (server, state) = build_ws_app();
    let eng_token = seed_user_and_login(&state, "eng_eq_flt", "pw", Role::Engineer);
    let mus_token = seed_user_and_login(&state, "mus_eq_flt", "pw", Role::Musician);

    {
        let (user_id, _, _, _) = state.db.find_user("mus_eq_flt").unwrap();
        state.db.assign_mix(0, user_id).unwrap();
    }

    let mut eng = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{eng_token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    let mut mus = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{mus_token}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;

    // Prime musician to prove it's in the receive loop.
    mus.send_text(ws_envelope(
        "SetEqBand",
        json!({
            "mix_index": 0, "band_index": 0,
            "frequency_hz": 1000.0, "gain_db": 0.0, "q": 1.0, "enabled": false
        }),
    ))
    .await;
    let prime_resp: Value = mus.receive_json().await;
    assert_eq!(prime_resp["payload"]["type"], "Error"); // FORBIDDEN

    eng.send_text(ws_envelope(
        "SetEqBand",
        json!({
            "mix_index": 0,
            "band_index": 0,
            "frequency_hz": 500.0,
            "gain_db": 3.0,
            "q": 1.0,
            "enabled": true
        }),
    ))
    .await;
    let eng_resp: Value = eng.receive_json().await;
    assert_eq!(eng_resp["payload"]["type"], "EqBandAck");

    // Musician must NOT receive EQ band broadcast.
    let mus_no_recv: Result<_, _> = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        mus.receive_json::<Value>(),
    )
    .await;
    assert!(
        mus_no_recv.is_err(),
        "musician must NOT receive EQ band broadcast"
    );
}

// End-to-end musician + simulated audio interface coverage.
#[tokio::test]
async fn musician_audio_simulation_full_http_cycle() {
    let (server, state) = build_test_app();
    let engineer = seed_user_and_login(&state, "eng_audio_cycle", "pw", Role::Engineer);
    let musician = seed_user_and_login(&state, "mus_audio_cycle", "pw", Role::Musician);
    let (musician_id, _, _, _) = state.db.find_user("mus_audio_cycle").unwrap();

    server
        .post("/api/v1/mixes/0/assign")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&engineer)
        .json(&json!({"user_id": musician_id}))
        .await
        .assert_status_ok();

    let gain: Value = server
        .put("/api/v1/mixes/0/sends/0/gain")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&musician)
        .json(&json!({"gain_db": -18.0}))
        .await
        .json();
    assert_eq!(gain["gain_db"], -18.0);

    let mute: Value = server
        .put("/api/v1/mixes/0/sends/0/mute")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&musician)
        .json(&json!({"muted": true}))
        .await
        .json();
    assert_eq!(mute["muted"], true);

    let send: Value = server
        .get("/api/v1/mixes/0/sends/0")
        .authorization_bearer(&musician)
        .await
        .json();
    assert_eq!(send["gain_db"], -18.0);
    assert_eq!(send["muted"], true);
}

#[tokio::test]
async fn musician_audio_simulation_webrtc_and_telemetry_contract() {
    let (server, state) = build_test_app();
    let engineer = seed_user_and_login(&state, "eng_audio_contract", "pw", Role::Engineer);
    let musician = seed_user_and_login(&state, "mus_audio_contract", "pw", Role::Musician);
    let (musician_id, _, _, _) = state.db.find_user("mus_audio_contract").unwrap();

    server
        .post("/api/v1/mixes/0/assign")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&engineer)
        .json(&json!({"user_id": musician_id}))
        .await
        .assert_status_ok();

    let offer: Value = server
        .post("/api/v1/audio/offer")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&musician)
        .json(&json!({"sdp": VALID_AUDIO_OFFER, "mix_id": null}))
        .await
        .json();
    assert!(offer["sdp"]
        .as_str()
        .is_some_and(|sdp| sdp.starts_with("v=0")));

    let telemetry: Value = server
        .get("/api/v1/telemetry")
        .authorization_bearer(&engineer)
        .await
        .json();
    assert_eq!(telemetry["backend"], "simulated");
    assert_eq!(telemetry["availability"], "simulated");

    server
        .get("/api/v1/telemetry")
        .authorization_bearer(&musician)
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn musician_audio_simulation_ws_ownership_is_enforced() {
    let (server, state) = build_ws_app();
    let musician = seed_user_and_login(&state, "mus_audio_ws", "pw", Role::Musician);
    let (musician_id, _, _, _) = state.db.find_user("mus_audio_ws").unwrap();
    state.db.assign_mix(0, musician_id).unwrap();

    let mut ws = server
        .get_websocket("/ws/v1")
        .add_header("Origin", "http://localhost")
        .add_header(
            "Sec-WebSocket-Protocol",
            format!("openiem.bearer.{musician}, openiem.v1"),
        )
        .await
        .into_websocket()
        .await;
    ws.send_text(ws_envelope(
        "SetSendGain",
        json!({"mix_index": 0, "channel_index": 0, "gain_db": -6.0}),
    ))
    .await;
    let ack: Value = ws.receive_json().await;
    assert_eq!(ack["payload"]["type"], "SendAck");
    assert_eq!(ack["payload"]["data"]["gain_db"], -6.0);

    ws.send_text(ws_envelope(
        "SetSendGain",
        json!({"mix_index": 1, "channel_index": 0, "gain_db": -6.0}),
    ))
    .await;
    let denied: Value = ws.receive_json().await;
    assert_eq!(denied["payload"]["type"], "Error");
    assert_eq!(denied["payload"]["data"]["code"], "FORBIDDEN");
}

// ── P1-008 domain routes ───────────────────────────────────────────────────

#[tokio::test]
async fn channels_list_returns_configured_channels() {
    let (server, state) = build_test_app();
    {
        let mut control = state.control.lock().expect("control lock");
        control
            .set_channel(0, Channel::new(1, "VOC 1"))
            .expect("channel 0 must configure");
        control
            .set_channel(1, Channel::new(2, "VOC 2"))
            .expect("channel 1 must configure");
    }
    let auth_credential = seed_user_and_login(&state, "eng_channels", "pw", Role::Engineer);
    let response = server
        .get("/api/v1/channels")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(auth_credential)
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert!(body["channels"]
        .as_array()
        .is_some_and(|channels| channels.len() >= 2));
}

#[tokio::test]
async fn system_info_returns_200() {
    let (server, _state) = build_test_app();
    let response = server.get("/api/v1/system").await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert!(!body["version"].as_str().unwrap_or_default().is_empty());
    assert_eq!(body["backend_status"], "SIMULATED");
}

#[tokio::test]
async fn channels_list_requires_auth() {
    let (server, _state) = build_test_app();
    server
        .get("/api/v1/channels")
        .add_header("Origin", "http://localhost")
        .await
        .assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ── /api/v1/presets ───────────────────────────────────────────────────────

#[tokio::test]
async fn presets_requires_authenticated_musician_role() {
    let (server, state) = build_test_app();
    let musician_token = seed_user_and_login(&state, "mus_presets_role", "pw", Role::Musician);
    server
        .get("/api/v1/presets")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(musician_token)
        .await
        .assert_status_ok();

    let response = server
        .get("/api/v1/presets")
        .add_header("Origin", "http://localhost")
        .await;
    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn engineer_can_list_read_only_presets() {
    let (server, state) = build_test_app();
    let engineer_token = seed_user_and_login(&state, "eng_presets_catalog", "pw", Role::Engineer);
    let response = server
        .get("/api/v1/presets")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(engineer_token)
        .await;
    response.assert_status_ok();
    let body: Value = response.json();
    let presets = body["presets"]
        .as_array()
        .expect("presets must be an array");
    assert_eq!(presets.len(), 2);
    assert_eq!(
        presets[0],
        json!({
            "id": "default-vocal",
            "name": "Vocal — Default",
            "kind": "channel",
            "description": "Safe neutral starting point for vocal channels."
        })
    );
    assert_eq!(presets[1]["id"], "default-instrument");
}

#[tokio::test]
async fn admin_can_list_read_only_presets() {
    let (server, state) = build_test_app();
    let admin_token = seed_user_and_login(&state, "admin_presets_catalog", "pw", Role::Admin);
    server
        .get("/api/v1/presets")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(admin_token)
        .await
        .assert_status_ok();
}

// ── /api/v1/metrics ───────────────────────────────────────────────────────

#[tokio::test]
async fn metrics_requires_engineer_role() {
    let (server, state) = build_test_app();
    let musician_token = seed_user_and_login(&state, "mus_metrics_role", "pw", Role::Musician);
    server
        .get("/api/v1/metrics")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(musician_token)
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn metrics_returns_schema_version_one() {
    let (server, state) = build_test_app();
    let engineer_token = seed_user_and_login(&state, "eng_metrics_schema", "pw", Role::Engineer);
    let resp = server
        .get("/api/v1/metrics")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(engineer_token)
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["schema_version"], 1);
}

#[tokio::test]
async fn metrics_exposes_receiver_counters() {
    let (server, state) = build_test_app();
    state.metrics.receiver.record_received();
    state.metrics.receiver.record_dropped();
    state.metrics.receiver.record_reconnect();
    state.metrics.receiver.record_plc_frame(3);
    state.metrics.receiver.record_output_failure();
    state.metrics.receiver.record_late();

    let engineer_token = seed_user_and_login(&state, "eng_metrics_receiver", "pw", Role::Engineer);
    let resp = server
        .get("/api/v1/metrics")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(engineer_token)
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["receiver"]["packets_received"], 1);
    assert_eq!(body["receiver"]["packets_dropped"], 1);
    assert_eq!(body["receiver"]["reconnect_count"], 1);
    assert_eq!(body["receiver"]["plc_frames_total"], 1);
    assert_eq!(body["receiver"]["plc_consecutive_max"], 3);
    assert_eq!(body["receiver"]["output_failures"], 1);
    assert_eq!(body["receiver"]["late_packets"], 1);
}

#[tokio::test]
async fn metrics_counters_start_at_zero() {
    let (server, state) = build_test_app();
    let engineer_token = seed_user_and_login(&state, "eng_metrics_zero", "pw", Role::Engineer);
    let resp = server
        .get("/api/v1/metrics")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(engineer_token)
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["audio"]["xrun_count"], 0);
    assert_eq!(body["audio"]["frames_processed"], 0);
    assert_eq!(body["stream"]["frames_sent"], 0);
    assert_eq!(body["stream"]["frames_lost"], 0);
    assert_eq!(body["stream"]["frames_plc_recovered"], 0);
    assert_eq!(body["receiver"]["packets_received"], 0);
    assert_eq!(body["receiver"]["packets_dropped"], 0);
    assert_eq!(body["receiver"]["output_failures"], 0);
    assert_eq!(body["receiver"]["late_packets"], 0);
    assert_eq!(body["network"]["late_packets"], 0);
}

// ── /api/v1/audio/pairing ─────────────────────────────────────────────────

/// Build a valid base64-encoded credential (at least 16 bytes).
fn make_credential(raw: &str) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(raw.as_bytes())
}

#[tokio::test]
async fn pair_device_requires_engineer_role() {
    let (server, state) = build_test_app();
    let musician_token = seed_user_and_login(&state, "mus_pair_role", "pw", Role::Musician);
    let resp = server
        .post("/api/v1/audio/pairing")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(musician_token)
        .json(&json!({
            "device_id": "rx-pair-role",
            "musician_id": "musician-1",
            "mix_index": 0,
            "credential": make_credential("pairing-secret-1234")
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn pair_device_engineer_succeeds() {
    let (server, state) = build_test_app();
    let engineer_token = seed_user_and_login(&state, "eng_pair_ok", "pw", Role::Engineer);
    let resp = server
        .post("/api/v1/audio/pairing")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&engineer_token)
        .json(&json!({
            "device_id": "rx-eng-ok",
            "musician_id": "musician-1",
            "mix_index": 0,
            "credential": make_credential("pairing-secret-1234")
        }))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["device_id"], "rx-eng-ok");
    assert_eq!(body["mix_index"], 0);
}

#[tokio::test]
async fn revoke_device_succeeds() {
    let (server, state) = build_test_app();
    let engineer_token = seed_user_and_login(&state, "eng_revoke_ok", "pw", Role::Engineer);
    // First pair the device.
    server
        .post("/api/v1/audio/pairing")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&engineer_token)
        .json(&json!({
            "device_id": "rx-revoke-ok",
            "musician_id": "musician-1",
            "mix_index": 0,
            "credential": make_credential("pairing-secret-1234")
        }))
        .await
        .assert_status_ok();
    // Now revoke it.
    let resp = server
        .delete("/api/v1/audio/pairing/rx-revoke-ok")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&engineer_token)
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["revoked"], true);
}

#[tokio::test]
async fn revoke_nonexistent_device_returns_404() {
    let (server, state) = build_test_app();
    let engineer_token = seed_user_and_login(&state, "eng_revoke_404", "pw", Role::Engineer);
    let resp = server
        .delete("/api/v1/audio/pairing/no-such-device")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&engineer_token)
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn offer_without_pairing_fields_is_backward_compatible() {
    let (server, state) = build_test_app();
    let musician_token = seed_user_and_login(&state, "mus_compat", "pw", Role::Musician);
    let resp = server
        .post("/api/v1/audio/offer")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(musician_token)
        .json(&json!({"sdp": VALID_AUDIO_OFFER, "mix_id": null}))
        .await;
    resp.assert_status_ok();
    let body: Value = resp.json();
    assert!(!body["sdp"].as_str().unwrap_or_default().is_empty());
}

#[tokio::test]
async fn offer_rejects_partial_pairing_fields() {
    let (server, state) = build_test_app();
    let musician_token = seed_user_and_login(&state, "mus_partial_pairing", "pw", Role::Musician);

    for payload in [
        json!({"sdp": VALID_AUDIO_OFFER, "mix_id": null, "device_id": "rx-partial"}),
        json!({"sdp": VALID_AUDIO_OFFER, "mix_id": null, "credential": make_credential("partial-secret-1234")}),
    ] {
        let resp = server
            .post("/api/v1/audio/offer")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(&musician_token)
            .json(&payload)
            .await;
        resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn offer_with_invalid_credential_returns_401() {
    let (server, state) = build_test_app();
    let engineer_token = seed_user_and_login(&state, "eng_cred_401", "pw", Role::Engineer);
    let musician_token = seed_user_and_login(&state, "mus_cred_401", "pw", Role::Musician);
    // Pair the device with a known credential.
    server
        .post("/api/v1/audio/pairing")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&engineer_token)
        .json(&json!({
            "device_id": "rx-cred-401",
            "musician_id": "musician-1",
            "mix_index": 0,
            "credential": make_credential("correct-secret-1234")
        }))
        .await
        .assert_status_ok();
    // Try offer with wrong credential.
    let resp = server
        .post("/api/v1/audio/offer")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(musician_token)
        .json(&json!({
            "sdp": VALID_AUDIO_OFFER,
            "mix_id": null,
            "device_id": "rx-cred-401",
            "credential": make_credential("wrong-secret-1234567")
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn offer_with_revoked_device_returns_403() {
    let (server, state) = build_test_app();
    let engineer_token = seed_user_and_login(&state, "eng_revoke_403", "pw", Role::Engineer);
    let musician_token = seed_user_and_login(&state, "mus_revoke_403", "pw", Role::Musician);
    let cred = make_credential("pairing-secret-1234");
    // Pair device.
    server
        .post("/api/v1/audio/pairing")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&engineer_token)
        .json(&json!({
            "device_id": "rx-revoke-403",
            "musician_id": "musician-1",
            "mix_index": 0,
            "credential": &cred
        }))
        .await
        .assert_status_ok();
    // Revoke it.
    server
        .delete("/api/v1/audio/pairing/rx-revoke-403")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(&engineer_token)
        .await
        .assert_status_ok();
    // Offer with revoked device must be 403.
    let resp = server
        .post("/api/v1/audio/offer")
        .add_header("Origin", "http://localhost")
        .authorization_bearer(musician_token)
        .json(&json!({
            "sdp": VALID_AUDIO_OFFER,
            "mix_id": null,
            "device_id": "rx-revoke-403",
            "credential": &cred
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}
