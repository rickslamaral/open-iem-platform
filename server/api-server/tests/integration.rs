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
        audio::{ice_candidate, offer, sessions},
        auth::{create_user, login, logout, refresh},
        channels::{get_state, set_channel_gain, set_channel_mute},
        health::health,
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
use serde_json::{Value, json};

// ── Test-only Ed25519 PEM pair ──────────────────────────────────────────────
// Generated once with `openssl genpkey -algorithm ed25519`. These keys are
// public test-only fixtures; they are never used for production tokens.

const TEST_PRIVATE_PEM: &[u8] =
    include_bytes!("fixtures/test_ed25519_private.pem");

const TEST_PUBLIC_PEM: &[u8] =
    include_bytes!("fixtures/test_ed25519_public.pem");

// ── Shared test fixture ────────────────────────────────────────────────────

fn build_test_app() -> (TestServer, AppState) {
    let jwt = JwtKeys::from_ed_pem(TEST_PRIVATE_PEM, TEST_PUBLIC_PEM)
        .expect("test PEM must be valid");
    let db = Db::open_in_memory().expect("in-memory DB must open");
    let state = AppState::new(ControlState::new(), db, jwt);

    let protected = Router::new()
        .route("/api/v1/state", get(get_state))
        .route("/api/v1/audio/offer", post(offer))
        .route("/api/v1/audio/ice-candidate", post(ice_candidate))
        .route("/api/v1/audio/sessions", get(sessions))
        .route("/api/v1/channels/{index}/gain", put(set_channel_gain))
        .route("/api/v1/channels/{index}/mute", put(set_channel_mute))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/admin/users", post(create_user))
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
    // Issue a token directly to avoid HTTP round-trip for token-endpoint tests.
    // For login-route tests we POST to /api/v1/auth/login instead.
    state
        .jwt
        .issue(username, role, &uuid::Uuid::new_v4().to_string())
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
