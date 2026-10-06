//! Auth routes: `POST /api/v1/auth/login`, `POST /api/v1/auth/refresh`, `POST /api/v1/auth/logout`.

#![allow(clippy::unused_async)]

use crate::{
    auth::{
        generate_refresh_token, hash_password, token_to_storage_key, verify_password, JwtClaims,
        REFRESH_TOKEN_TTL_S,
    },
    error::ApiError,
    middleware::require_min_role,
    state::AppState,
};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use control_protocol::Role;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const MAX_USERNAME_BYTES: usize = 128;
const MIN_PASSWORD_BYTES: usize = 8;
const MAX_PASSWORD_BYTES: usize = 1024;

fn validate_credentials(username: &str, password: &str) -> Result<(), ApiError> {
    if username.is_empty() || username.len() > MAX_USERNAME_BYTES {
        return Err(ApiError::BadRequest(
            "username length is invalid".to_owned(),
        ));
    }
    if password.len() < MIN_PASSWORD_BYTES || password.len() > MAX_PASSWORD_BYTES {
        return Err(ApiError::BadRequest(
            "password length is invalid".to_owned(),
        ));
    }
    Ok(())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Build refresh cookie attributes.
///
/// Local installs use HTTP by default, including Raspberry Pi and desktop
/// deployments. Production hardening opts into `Secure` explicitly with
/// `OPENIEM_REQUIRE_SECURE_COOKIES=true`.
pub(crate) fn refresh_cookie(raw: &str, max_age: u64) -> String {
    let secure = if std::env::var("OPENIEM_REQUIRE_SECURE_COOKIES").as_deref() == Ok("true") {
        " Secure;"
    } else {
        ""
    };
    format!(
        "refresh_token={raw}; HttpOnly;{secure} SameSite=Strict; Path=/api/v1/auth/refresh; Max-Age={max_age}"
    )
}

/// Parse a named cookie from the `Cookie` request header.
/// Returns the value as `&str` if found.
fn extract_cookie_value<'a>(headers: &'a axum::http::HeaderMap, name: &str) -> Option<&'a str> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some((k, v)) = pair.split_once('=') {
            if k.trim() == name {
                return Some(v.trim());
            }
        }
    }
    None
}

/// Login request body.
#[derive(Deserialize)]
pub struct LoginRequest {
    /// Username.
    pub username: String,
    /// Plaintext password (transmitted over HTTPS only).
    pub password: String,
}

/// Login response body.
///
/// The refresh token is NOT returned here — it is delivered exclusively via
/// `Set-Cookie: refresh_token=...; HttpOnly; Secure; SameSite=Strict` to
/// prevent JavaScript access. The access token is short-lived (15 min) and
/// is safe to return in the body.
#[derive(Serialize)]
pub struct LoginResponse {
    /// Short-lived access token (JWT, 15 min). Store in memory only — not localStorage.
    pub access_token: String,
    /// Role granted.
    pub role: Role,
    /// Whether first-access password change is mandatory.
    pub must_change_password: bool,
}

/// `POST /api/v1/auth/login`
///
/// Validates username + password, issues access + refresh tokens.
/// Refresh token is delivered via `HttpOnly` cookie only, not in the body.
///
/// # Errors
/// Returns `ApiError::Unauthorized` on bad credentials.
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validate_credentials(&body.username, &body.password)?;
    let _auth_guard = state
        .auth_lock
        .lock()
        .map_err(|_| ApiError::Internal("auth lock poisoned".to_owned()))?;
    let (user_id, pw_hash, role, must_change_password) = state.db.find_user(&body.username)?;
    verify_password(&body.password, &pw_hash)?;

    let jti = Uuid::new_v4().to_string();
    let raw_refresh = generate_refresh_token();
    let family = Uuid::new_v4().to_string();
    let now = unix_now();
    let expires_at = now + REFRESH_TOKEN_TTL_S;
    let token_hash = token_to_storage_key(&raw_refresh);
    let session_id = state.db.create_session_with_access(
        user_id,
        &token_hash,
        expires_at,
        &family,
        &jti,
        now + crate::auth::ACCESS_TOKEN_TTL_S,
    )?;
    let access =
        match state
            .jwt
            .issue_with_session(&body.username, user_id, role, &jti, Some(session_id))
        {
            Ok(access) => access,
            Err(error) => {
                state
                    .db
                    .cleanup_session_after_signing_failure(&token_hash, &jti)?;
                return Err(error);
            }
        };

    Ok((
        StatusCode::OK,
        [(
            header::SET_COOKIE,
            refresh_cookie(&raw_refresh, REFRESH_TOKEN_TTL_S),
        )],
        Json(LoginResponse {
            access_token: access,
            role,
            must_change_password,
        }),
    ))
}

/// Change password for the authenticated user, including first-access bootstrap.
///
/// # Errors
/// Returns `ApiError::BadRequest` for invalid password length or
/// `ApiError::Internal` when hashing or persisting the password fails.
pub async fn change_password(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let _auth_guard = state
        .auth_lock
        .lock()
        .map_err(|_| ApiError::Internal("auth lock poisoned".to_owned()))?;
    validate_credentials(&claims.sub, &body.new_password)?;
    let password_hash = hash_password(&body.new_password)?;
    state.db.change_password(claims.user_id, &password_hash)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Change-password request body.
#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    /// New plaintext password (transmitted over HTTPS only).
    pub new_password: String,
}

/// Refresh response body.
///
/// New refresh token is delivered via `HttpOnly` cookie only.
#[derive(Serialize)]
pub struct RefreshResponse {
    /// New access token.
    pub access_token: String,
}

/// `POST /api/v1/auth/refresh`
///
/// Reads refresh token from `Cookie: refresh_token=...` header.
/// Rotates refresh token and issues a new access token.
/// New refresh token is delivered via `HttpOnly` cookie only.
///
/// # Errors
/// Returns `ApiError::Unauthorized` on invalid, expired, or replayed refresh token.
pub async fn refresh(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let raw_refresh = extract_cookie_value(&headers, "refresh_token")
        .ok_or(ApiError::Unauthorized("missing refresh_token cookie"))?;
    let _refresh_guard = state
        .refresh_lock
        .lock()
        .map_err(|_| ApiError::Internal("refresh lock poisoned".to_owned()))?;

    let now = unix_now();
    let token_hash = token_to_storage_key(raw_refresh);
    // Resolve owner before rotation. A post-rotation lookup failure would
    // consume the refresh token without a usable replacement.
    let user_id = state.db.refresh_token_owner(&token_hash)?;
    let (username, role) = state.db.find_user_by_id(user_id)?;
    let jti = Uuid::new_v4().to_string();
    let raw_new_refresh = generate_refresh_token();
    let expires_at = now + REFRESH_TOKEN_TTL_S;
    let new_token_hash = token_to_storage_key(&raw_new_refresh);
    let (_, _, new_session_id) = state.db.rotate_refresh_token_with_access_id(
        &token_hash,
        &new_token_hash,
        expires_at,
        now,
        Some(&jti),
        Some(now + crate::auth::ACCESS_TOKEN_TTL_S),
    )?;
    let access =
        match state
            .jwt
            .issue_with_session(&username, user_id, role, &jti, Some(new_session_id))
        {
            Ok(access) => access,
            Err(error) => {
                state
                    .db
                    .discard_refresh_after_signing_failure(&new_token_hash, &jti)?;
                return Err(error);
            }
        };

    Ok((
        StatusCode::OK,
        [(
            header::SET_COOKIE,
            refresh_cookie(&raw_new_refresh, REFRESH_TOKEN_TTL_S),
        )],
        Json(RefreshResponse {
            access_token: access,
        }),
    ))
}

/// `POST /api/v1/auth/logout`
///
/// Revokes all refresh tokens for the authenticated user.
/// Requires valid access token (via JWT middleware).
///
/// # Errors
/// Returns `ApiError::Internal` on DB failure.
pub async fn logout(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    let (user_id, _, _, _) = state.db.find_user(&claims.sub)?;
    state.db.revoke_all_for_user(user_id)?;
    // Clear the cookie
    Ok((
        StatusCode::NO_CONTENT,
        [(
            header::SET_COOKIE,
            "refresh_token=; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth/refresh; Max-Age=0".to_owned(),
        )],
    ))
}

/// Admin-only: seed a new user account.
///
/// # Errors
/// Returns `ApiError::Forbidden` if caller is not Admin, or `ApiError::BadRequest` on duplicate.
pub async fn create_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Json(body): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Admin)?;
    validate_credentials(&body.username, &body.password)?;
    let pw_hash = hash_password(&body.password)?;
    state.db.create_user(&body.username, &pw_hash, body.role)?;
    Ok(StatusCode::CREATED)
}

/// Create user request body.
#[derive(Deserialize)]
pub struct CreateUserRequest {
    /// Username.
    pub username: String,
    /// Plaintext password.
    pub password: String,
    /// Role to assign.
    pub role: Role,
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{generate_refresh_token, hash_password, token_to_storage_key, JwtKeys},
        db::Db,
        middleware::jwt_auth,
        routes::auth::{change_password, create_user, login, logout, refresh},
        security::validate_origin,
        state::AppState,
    };
    use axum::{
        middleware,
        routing::{post, put},
        Router,
    };
    use axum_test::TestServer;
    use control_protocol::Role;
    use control_server::ControlState;
    use serde_json::json;
    use std::{fs, process::Command};
    use uuid::Uuid;

    fn test_keys() -> (Vec<u8>, Vec<u8>) {
        let nonce = Uuid::new_v4();
        let private_path = std::env::temp_dir().join(format!("open-iem-auth-test-{nonce}.pem"));
        let public_path = std::env::temp_dir().join(format!("open-iem-auth-test-{nonce}.pub.pem"));
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
        let control = ControlState::new();
        let state = AppState::new(control, db, jwt);

        let protected = Router::new()
            .route("/api/v1/auth/password", put(change_password))
            .route("/api/v1/auth/logout", post(logout))
            .route("/api/v1/admin/users", post(create_user))
            .layer(middleware::from_fn_with_state(state.clone(), jwt_auth));

        let public = Router::new()
            .route("/api/v1/auth/login", post(login))
            .route("/api/v1/auth/refresh", post(refresh));

        let app = Router::new()
            .merge(protected)
            .merge(public)
            .with_state(state.clone())
            .layer(middleware::from_fn(validate_origin));

        (TestServer::new(app), state)
    }

    fn seed_user_token(state: &AppState, username: &str, role: Role) -> (i64, String) {
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
        let token = state
            .jwt
            .issue_with_session(username, user_id, role, &jti, Some(session_id))
            .expect("token issued");
        (user_id, token)
    }

    // --- change_password ---

    #[tokio::test]
    async fn change_password_authenticated_ok() {
        let (server, state) = build_test_app();
        // change_password requires must_change_password=1; use bootstrap soundtech user
        state
            .db
            .bootstrap_soundtech("initialpass")
            .expect("bootstrap soundtech");
        let (user_id, _, _, _) = state.db.find_user("soundtech").expect("soundtech exists");
        let jti = uuid::Uuid::new_v4().to_string();
        let raw_refresh = crate::auth::generate_refresh_token();
        let session_id = state
            .db
            .create_session_with_access(
                user_id,
                &crate::auth::token_to_storage_key(&raw_refresh),
                4_000_000_000,
                &uuid::Uuid::new_v4().to_string(),
                &jti,
                4_000_000_000,
            )
            .expect("session created");
        let token = state
            .jwt
            .issue_with_session(
                "soundtech",
                user_id,
                control_protocol::Role::Engineer,
                &jti,
                Some(session_id),
            )
            .expect("token issued");
        let resp = server
            .put("/api/v1/auth/password")
            .authorization_bearer(token)
            .json(&json!({"new_password": "newpassword123"}))
            .await;
        resp.assert_status(axum::http::StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn change_password_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server
            .put("/api/v1/auth/password")
            .json(&json!({"new_password": "newpassword123"}))
            .await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn change_password_short_password_rejected() {
        let (server, state) = build_test_app();
        // validate_credentials runs before db.change_password — short password fails with 400
        // regardless of must_change_password flag; use any authenticated user
        let (_id, token) = seed_user_token(&state, "user2", Role::Musician);
        let resp = server
            .put("/api/v1/auth/password")
            .authorization_bearer(token)
            .json(&json!({"new_password": "short"}))
            .await;
        resp.assert_status_bad_request();
    }

    // --- create_user ---

    #[tokio::test]
    async fn create_user_admin_ok() {
        let (server, state) = build_test_app();
        let (_id, token) = seed_user_token(&state, "admin1", Role::Admin);
        let resp = server
            .post("/api/v1/admin/users")
            .authorization_bearer(token)
            .json(
                &json!({"username": "newuser1", "password": "securepassword", "role": "MUSICIAN"}),
            )
            .await;
        resp.assert_status(axum::http::StatusCode::CREATED);
    }

    #[tokio::test]
    async fn create_user_engineer_forbidden() {
        let (server, state) = build_test_app();
        let (_id, token) = seed_user_token(&state, "eng1", Role::Engineer);
        let resp = server
            .post("/api/v1/admin/users")
            .authorization_bearer(token)
            .json(
                &json!({"username": "newuser2", "password": "securepassword", "role": "MUSICIAN"}),
            )
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn create_user_requires_authentication() {
        let (server, _state) = build_test_app();
        let resp = server
            .post("/api/v1/admin/users")
            .json(
                &json!({"username": "newuser3", "password": "securepassword", "role": "MUSICIAN"}),
            )
            .await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn create_user_empty_username_rejected() {
        let (server, state) = build_test_app();
        let (_id, token) = seed_user_token(&state, "admin2", Role::Admin);
        let resp = server
            .post("/api/v1/admin/users")
            .authorization_bearer(token)
            .json(&json!({"username": "", "password": "securepassword", "role": "MUSICIAN"}))
            .await;
        resp.assert_status_bad_request();
    }

    // --- login ---

    #[tokio::test]
    async fn login_valid_credentials_ok() {
        let (server, state) = build_test_app();
        let pw_hash = hash_password("correct_pw").expect("hash");
        state
            .db
            .create_user("login_ok", &pw_hash, Role::Musician)
            .expect("create");

        let resp = server
            .post("/api/v1/auth/login")
            .add_header("Origin", "http://localhost")
            .json(&json!({"username": "login_ok", "password": "correct_pw"}))
            .await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert!(body.get("access_token").is_some(), "access_token absent");
    }

    #[tokio::test]
    async fn login_wrong_password_returns_unauthorized() {
        let (server, state) = build_test_app();
        let pw_hash = hash_password("correct_pw").expect("hash");
        state
            .db
            .create_user("login_badpw", &pw_hash, Role::Musician)
            .expect("create");

        let resp = server
            .post("/api/v1/auth/login")
            .add_header("Origin", "http://localhost")
            .json(&json!({"username": "login_badpw", "password": "wrong_pw"}))
            .await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn login_unknown_user_returns_not_found() {
        let (server, _state) = build_test_app();

        let resp = server
            .post("/api/v1/auth/login")
            .add_header("Origin", "http://localhost")
            .json(&json!({"username": "nobody_here", "password": "any_pass"}))
            .await;
        // find_user returns NotFound which maps to 404
        assert!(
            resp.status_code().as_u16() == 404 || resp.status_code().as_u16() == 401,
            "expected 404 or 401 for unknown user, got {}",
            resp.status_code()
        );
    }

    #[tokio::test]
    async fn login_empty_username_returns_bad_request() {
        let (server, _state) = build_test_app();

        let resp = server
            .post("/api/v1/auth/login")
            .add_header("Origin", "http://localhost")
            .json(&json!({"username": "", "password": "some_pw"}))
            .await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn login_short_password_returns_bad_request() {
        let (server, _state) = build_test_app();

        let resp = server
            .post("/api/v1/auth/login")
            .add_header("Origin", "http://localhost")
            .json(&json!({"username": "someone", "password": "short"}))
            .await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn login_empty_password_returns_bad_request() {
        let (server, _state) = build_test_app();

        let resp = server
            .post("/api/v1/auth/login")
            .add_header("Origin", "http://localhost")
            .json(&json!({"username": "someone", "password": ""}))
            .await;
        resp.assert_status_bad_request();
    }

    // --- refresh ---

    #[tokio::test]
    async fn refresh_missing_cookie_returns_unauthorized() {
        let (server, _state) = build_test_app();

        let resp = server
            .post("/api/v1/auth/refresh")
            .add_header("Origin", "http://localhost")
            .await;
        resp.assert_status_unauthorized();
    }

    // --- logout ---

    #[tokio::test]
    async fn logout_requires_authentication() {
        let (server, _state) = build_test_app();

        let resp = server
            .post("/api/v1/auth/logout")
            .add_header("Origin", "http://localhost")
            .await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn logout_authenticated_returns_no_content() {
        let (server, state) = build_test_app();
        let (_id, token) = seed_user_token(&state, "logout_user", Role::Musician);

        let resp = server
            .post("/api/v1/auth/logout")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .await;
        resp.assert_status(axum::http::StatusCode::NO_CONTENT);
    }
}
