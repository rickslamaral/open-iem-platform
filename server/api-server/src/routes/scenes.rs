//! Scene management REST API.
//!
//! GET    /api/v1/scenes           — list all scenes
//! POST   /api/v1/scenes           — create scene (Engineer/Admin)
//! GET    /api/v1/scenes/active    — get active scene
//! GET    /api/v1/scenes/{id}      — get scene by ID
//! PUT    /api/v1/scenes/{id}      — save new revision (Engineer/Admin)
//! DELETE /api/v1/scenes/{id}      — delete scene (Engineer/Admin)
//! GET    /api/v1/scenes/backup — export durable scenes
//! PUT    /api/v1/scenes/backup — replace durable scenes from export
//! POST   /api/v1/scenes/{id}/recall — set as active (Engineer/Admin)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use control_protocol::Role;
use scene_manager::StoreError;
use serde::{Deserialize, Serialize};

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// Request body for `POST /api/v1/scenes`.
#[derive(Deserialize)]
pub struct CreateSceneRequest {
    /// Scene display name.
    pub name: String,
    /// Initial scene configuration.
    pub config: scene_manager::SceneConfig,
}

/// Request body for `PUT /api/v1/scenes/{id}`.
#[derive(Deserialize)]
pub struct UpdateSceneRequest {
    /// Replacement scene configuration.
    pub config: scene_manager::SceneConfig,
}

/// Summary entry in the list response.
#[derive(Serialize)]
pub struct SceneSummaryDto {
    /// Stable scene ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Current active revision number.
    pub active_revision: u64,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last update.
    pub updated_at: i64,
}

/// Response body for `GET /api/v1/scenes`.
#[derive(Serialize)]
pub struct ListScenesResponse {
    /// All scenes in creation order.
    pub scenes: Vec<SceneSummaryDto>,
}

/// Response body for `GET /api/v1/scenes/active`.
#[derive(Serialize)]
pub struct ActiveSceneResponse {
    /// Currently active scene, if any.
    pub scene: Option<scene_manager::Scene>,
}

/// `GET /api/v1/scenes/backup` — exports durable scene state.
#[allow(clippy::unused_async)]
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Engineer role.
/// Returns `ApiError::Internal` on store failure.
pub async fn backup_scenes(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let snapshot = state.scenes.export_snapshot().map_err(map_store_err)?;
    Ok(([("Content-Type", "application/json")], snapshot))
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn map_store_err(e: StoreError) -> ApiError {
    match e {
        StoreError::NotFound(_) => ApiError::NotFound(e.to_string()),
        StoreError::IsActive => ApiError::BadRequest("scene is active".to_owned()),
        StoreError::Validation(ve) => ApiError::BadRequest(ve.to_string()),
        StoreError::CorruptPayload(msg) => ApiError::Internal(msg),
        StoreError::Db(dbe) => ApiError::Internal(dbe.to_string()),
        StoreError::LockPoisoned => ApiError::Internal("scene store lock poisoned".to_owned()),
        StoreError::InvalidSnapshot(msg) => ApiError::BadRequest(msg),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `PUT /api/v1/scenes/backup` — restores a complete durable scene snapshot.
///
/// Restore is validated before replacement and commits atomically. Runtime
/// state is never accepted by the strict snapshot schema.
///
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Engineer role,
/// `ApiError::BadRequest` for invalid snapshots, or `ApiError::Internal`
/// when persistence fails.
#[allow(clippy::unused_async)]
pub async fn restore_scenes(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Json(snapshot): Json<scene_manager::SceneStoreSnapshot>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let json = serde_json::to_string(&snapshot)
        .map_err(|e| ApiError::BadRequest(format!("invalid scene snapshot: {e}")))?;
    state
        .scenes
        .restore_snapshot(&json)
        .map_err(map_store_err)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/v1/scenes` — list all scenes.
#[allow(clippy::unused_async)]
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Musician role.
/// Returns `ApiError::Internal` on store failure.
pub async fn list_scenes(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Musician)?;
    let summaries = state.scenes.list_scenes().map_err(map_store_err)?;
    let scenes = summaries
        .into_iter()
        .map(|s| SceneSummaryDto {
            id: s.id,
            name: s.name,
            active_revision: s.active_revision,
            created_at: s.created_at,
            updated_at: s.updated_at,
        })
        .collect();
    Ok(Json(ListScenesResponse { scenes }))
}

/// `POST /api/v1/scenes` — create a new scene.
#[allow(clippy::unused_async)]
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Engineer role.
/// Returns `ApiError::BadRequest` on validation failure.
/// Returns `ApiError::Internal` on store failure.
pub async fn create_scene(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Json(body): Json<CreateSceneRequest>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let scene = state
        .scenes
        .create_scene(&body.name, body.config)
        .map_err(map_store_err)?;
    Ok((StatusCode::CREATED, Json(scene)))
}

/// `GET /api/v1/scenes/active` — get the currently active scene.
#[allow(clippy::unused_async)]
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Musician role.
/// Returns `ApiError::Internal` on store failure.
pub async fn get_active_scene(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Musician)?;
    let scene = state.scenes.get_active_scene().map_err(map_store_err)?;
    Ok(Json(ActiveSceneResponse { scene }))
}

/// `GET /api/v1/scenes/{id}` — get a scene by ID.
#[allow(clippy::unused_async)]
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Musician role.
/// Returns `ApiError::NotFound` when `id` does not exist.
/// Returns `ApiError::Internal` on store failure.
pub async fn get_scene(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Musician)?;
    let scene = state.scenes.get_scene(&id).map_err(map_store_err)?;
    Ok(Json(scene))
}

/// `PUT /api/v1/scenes/{id}` — save a new revision.
#[allow(clippy::unused_async)]
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Engineer role.
/// Returns `ApiError::NotFound` when `id` does not exist.
/// Returns `ApiError::BadRequest` on validation failure.
/// Returns `ApiError::Internal` on store failure.
pub async fn update_scene(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(id): Path<String>,
    Json(body): Json<UpdateSceneRequest>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let scene = state
        .scenes
        .save_scene(&id, body.config)
        .map_err(map_store_err)?;
    Ok(Json(scene))
}

/// `DELETE /api/v1/scenes/{id}` — delete a scene.
#[allow(clippy::unused_async)]
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Engineer role.
/// Returns `ApiError::NotFound` when `id` does not exist.
/// Returns `ApiError::BadRequest` when scene is active.
/// Returns `ApiError::Internal` on store failure.
pub async fn delete_scene(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    state.scenes.delete_scene(&id).map_err(map_store_err)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/v1/scenes/{id}/recall` — set scene as active.
#[allow(clippy::unused_async)]
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Engineer role.
/// Returns `ApiError::NotFound` when `id` does not exist.
/// Returns `ApiError::Internal` on store failure.
pub async fn recall_scene(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    state
        .scenes
        .set_active_scene(Some(id.as_str()))
        .map_err(map_store_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Integration tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::{
        auth::{generate_refresh_token, hash_password, token_to_storage_key, JwtKeys},
        db::Db,
        middleware::jwt_auth,
        routes::scenes::{
            create_scene, delete_scene, get_active_scene, get_scene, list_scenes, recall_scene,
            update_scene,
        },
        security::validate_origin,
        state::AppState,
    };
    use axum::{
        middleware,
        routing::{get, post},
        Router,
    };
    use axum_test::TestServer;
    use control_protocol::Role;
    use control_server::ControlState;
    use serde_json::{json, Value};
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
        let private_path = std::env::temp_dir().join(format!("open-iem-scenes-test-{nonce}.pem"));
        let public_path =
            std::env::temp_dir().join(format!("open-iem-scenes-test-{nonce}.pub.pem"));
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
            .expect("openssl public-key export failed");
        assert!(status.success(), "openssl public-key export failed");

        let private = fs::read(&private_path).expect("generated private key must be readable");
        let public = fs::read(&public_path).expect("generated public key must be readable");
        let _ = fs::remove_file(private_path);
        let _ = fs::remove_file(public_path);
        (private, public)
    }

    fn build_test_app() -> (TestServer, AppState) {
        let (private_pem, public_pem) = test_keys();
        let jwt = JwtKeys::from_ed_pem(&private_pem, &public_pem).expect("test PEM must be valid");
        let db = Db::open_in_memory().expect("in-memory DB must open");
        let state = AppState::new(ControlState::new(), db, jwt);

        let protected = Router::new()
            .route("/api/v1/scenes", get(list_scenes).post(create_scene))
            .route("/api/v1/scenes/active", get(get_active_scene))
            .route(
                "/api/v1/scenes/{id}",
                get(get_scene).put(update_scene).delete(delete_scene),
            )
            .route("/api/v1/scenes/{id}/recall", post(recall_scene))
            .layer(middleware::from_fn_with_state(state.clone(), jwt_auth));

        let app = Router::new()
            .merge(protected)
            .with_state(state.clone())
            .layer(middleware::from_fn(validate_origin));

        let server = TestServer::new(app);
        (server, state)
    }

    fn seed_user_and_login(state: &AppState, username: &str, password: &str, role: Role) -> String {
        let pw_hash = hash_password(password).expect("hash must succeed");
        state
            .db
            .create_user(username, &pw_hash, role)
            .expect("create user must succeed");
        let (user_id, _, _, _) = state.db.find_user(username).expect("user must exist");
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

    fn empty_scene_body() -> Value {
        json!({
            "name": "Test Show",
            "config": {
                "channels": [],
                "mixes": []
            }
        })
    }

    #[tokio::test]
    async fn list_scenes_empty() {
        let (server, state) = build_test_app();
        let token = seed_user_and_login(&state, "mus_list_empty", "pw", Role::Musician);
        let resp = server
            .get("/api/v1/scenes")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["scenes"], json!([]));
    }

    #[tokio::test]
    async fn create_scene_engineer_ok() {
        let (server, state) = build_test_app();
        let token = seed_user_and_login(&state, "eng_create_ok", "pw", Role::Engineer);
        let resp = server
            .post("/api/v1/scenes")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .json(&empty_scene_body())
            .await;
        resp.assert_status(axum::http::StatusCode::CREATED);
        let body: Value = resp.json();
        assert_eq!(body["name"], "Test Show");
        assert_eq!(body["revision"], 1);
        assert!(body["id"].as_str().is_some());
    }

    #[tokio::test]
    async fn create_scene_musician_forbidden() {
        let (server, state) = build_test_app();
        let token = seed_user_and_login(&state, "mus_create_forbidden", "pw", Role::Musician);
        server
            .post("/api/v1/scenes")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .json(&empty_scene_body())
            .await
            .assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn get_scene_by_id_ok() {
        let (server, state) = build_test_app();
        let eng_token = seed_user_and_login(&state, "eng_get_id_ok", "pw", Role::Engineer);
        // Create a scene first
        let create_resp = server
            .post("/api/v1/scenes")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(eng_token.clone())
            .json(&empty_scene_body())
            .await;
        create_resp.assert_status(axum::http::StatusCode::CREATED);
        let scene_id = create_resp.json::<Value>()["id"]
            .as_str()
            .unwrap()
            .to_owned();

        let mus_token = seed_user_and_login(&state, "mus_get_id_ok", "pw", Role::Musician);
        let resp = server
            .get(&format!("/api/v1/scenes/{scene_id}"))
            .add_header("Origin", "http://localhost")
            .authorization_bearer(mus_token)
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["id"], scene_id);
        assert_eq!(body["name"], "Test Show");
    }

    #[tokio::test]
    async fn get_scene_not_found() {
        let (server, state) = build_test_app();
        let token = seed_user_and_login(&state, "eng_get_nf", "pw", Role::Engineer);
        server
            .get("/api/v1/scenes/nonexistent-id-xyz")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .await
            .assert_status(axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn recall_scene_sets_active() {
        let (server, state) = build_test_app();
        let token = seed_user_and_login(&state, "eng_recall", "pw", Role::Engineer);

        // Create scene
        let create_resp = server
            .post("/api/v1/scenes")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token.clone())
            .json(&empty_scene_body())
            .await;
        create_resp.assert_status(axum::http::StatusCode::CREATED);
        let scene_id = create_resp.json::<Value>()["id"]
            .as_str()
            .unwrap()
            .to_owned();

        // Recall
        server
            .post(&format!("/api/v1/scenes/{scene_id}/recall"))
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token.clone())
            .await
            .assert_status(axum::http::StatusCode::NO_CONTENT);

        // GET active
        let resp = server
            .get("/api/v1/scenes/active")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["scene"]["id"], scene_id);
    }

    #[tokio::test]
    async fn delete_active_scene_fails() {
        let (server, state) = build_test_app();
        let token = seed_user_and_login(&state, "eng_del_active", "pw", Role::Engineer);

        // Create + recall
        let create_resp = server
            .post("/api/v1/scenes")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token.clone())
            .json(&empty_scene_body())
            .await;
        create_resp.assert_status(axum::http::StatusCode::CREATED);
        let scene_id = create_resp.json::<Value>()["id"]
            .as_str()
            .unwrap()
            .to_owned();

        server
            .post(&format!("/api/v1/scenes/{scene_id}/recall"))
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token.clone())
            .await
            .assert_status(axum::http::StatusCode::NO_CONTENT);

        // Delete active scene → 400
        server
            .delete(&format!("/api/v1/scenes/{scene_id}"))
            .add_header("Origin", "http://localhost")
            .authorization_bearer(token)
            .await
            .assert_status(axum::http::StatusCode::BAD_REQUEST);
    }
}
