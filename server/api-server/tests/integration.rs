//! HTTP integration tests for api-server.
//!
//! Tests spin up a full Axum router with an in-memory SQLite database and
//! test-only Ed25519 keys. Every test exercises the complete middleware stack:
//! Origin/CSRF check → JWT auth → role check → handler.
//!
//! Test keys are generated at compile time from a fixed PEM pair that carries
//! no sensitive information (test-only, never used for real tokens).

use api_server::{
    auth::{hash_password, JwtKeys},
    db::Db,
    middleware::jwt_auth,
    routes::{
        admin::{
            delete_user as admin_delete_user, list_sessions as admin_list_sessions,
            list_users as admin_list_users, revoke_session as admin_revoke_session,
        },
        audio::{ice_candidate, offer, sessions},
        auth::{create_user, login, logout, refresh},
        channels::{get_state, set_channel_gain, set_channel_mute},
        health::health,
        mixes::{
            assign_mix, get_send_state, list_mixes, set_send_gain, set_send_muted, set_send_pan,
            unassign_mix,
        },
        telemetry::get_telemetry,
    },
    security::validate_origin,
    state::AppState,
    ws::ws_handler,
};
use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post, put},
    Router,
};
use axum_test::TestServer;
use control_protocol::Role;
use control_server::ControlState;
use mix_engine::Mix;
use serde_json::{json, Value};

// ── Test-only Ed25519 PEM pair ──────────────────────────────────────────────
// Generated once with `openssl genpkey -algorithm ed25519`. These keys are
// public test-only fixtures; they are never used for production tokens.

const TEST_PRIVATE_PEM: &[u8] = include_bytes!("fixtures/test_ed25519_private.pem");

const TEST_PUBLIC_PEM: &[u8] = include_bytes!("fixtures/test_ed25519_public.pem");

// ── Shared test fixture ────────────────────────────────────────────────────

fn build_test_app() -> (TestServer, AppState) {
    let jwt =
        JwtKeys::from_ed_pem(TEST_PRIVATE_PEM, TEST_PUBLIC_PEM).expect("test PEM must be valid");
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
        .route("/api/v1/telemetry", get(get_telemetry))
        .route("/api/v1/audio/offer", post(offer))
        .route("/api/v1/audio/ice-candidate", post(ice_candidate))
        .route("/api/v1/audio/sessions", get(sessions))
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
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/refresh", post(refresh));

    let app = Router::new()
        .merge(protected)
        .merge(public)
        .with_state(state.clone())
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
    let (user_id, _, _) = state.db.find_user(username).expect("user must exist");
    // Issue a token directly to avoid HTTP round-trip for token-endpoint tests;
    // For login-route tests we POST to /api/v1/auth/login instead.
    state
        .jwt
        .issue(username, user_id, role, &uuid::Uuid::new_v4().to_string())
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
    let (musician_id, _, _) = state.db.find_user("mus_mix").unwrap();

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
    let (id, _, _) = state.db.find_user("mus_invalid").unwrap();
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
    let (user_id, _, _) = state.db.find_user("todelete1").unwrap();
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
    let (caller_id, _, _) = state.db.find_user("admin_self").unwrap();
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
    let (user_id, _, _) = state.db.find_user("admin_sr2").unwrap();
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
