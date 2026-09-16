//! GET /api/v1/devices — device registry snapshot (Engineer/Admin only).

use axum::{extract::State, response::IntoResponse, Json};
use control_protocol::Role;
use device_manager::DeviceState;
use serde::Serialize;

use crate::{auth::JwtClaims, error::ApiError, middleware::require_min_role, state::AppState};

/// Device state string for JSON output.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStateDto {
    /// Device is present and usable.
    Available,
    /// Device disappeared or failed; output must be muted.
    Recovering,
    /// Device returned and awaits successful capability validation.
    Reconnected,
}

impl From<DeviceState> for DeviceStateDto {
    fn from(s: DeviceState) -> Self {
        match s {
            DeviceState::Available => Self::Available,
            DeviceState::Recovering => Self::Recovering,
            DeviceState::Reconnected => Self::Reconnected,
        }
    }
}

/// Capability snapshot for one device.
#[derive(Debug, Serialize)]
pub struct DeviceDto {
    /// Stable device identifier.
    pub id: String,
    /// Human-readable device name.
    pub name: String,
    /// Supported sample rates in Hz.
    pub sample_rates_hz: Vec<u32>,
    /// Maximum number of output channels.
    pub max_output_channels: usize,
    /// Current lifecycle state.
    pub state: DeviceStateDto,
}

/// Response body for GET /api/v1/devices.
#[derive(Debug, Serialize)]
pub struct DevicesResponse {
    /// All registered devices.
    pub devices: Vec<DeviceDto>,
    /// Whether at least one device is in the Available state.
    pub has_available_device: bool,
}

/// Handler for `GET /api/v1/devices`.
///
/// Returns a snapshot of the bounded device registry.
/// Requires at least Engineer role.
///
/// # Errors
/// Returns `ApiError::Forbidden` when caller lacks Engineer or Admin role.
/// Returns `StatusCode::INTERNAL_SERVER_ERROR` if the device lock is poisoned.
#[allow(clippy::unused_async)]
pub async fn get_devices(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<JwtClaims>,
) -> Result<impl IntoResponse, ApiError> {
    require_min_role(&claims, Role::Engineer)?;
    let mgr = state
        .devices
        .lock()
        .map_err(|_| ApiError::Internal("device registry lock poisoned".to_owned()))?;
    let devices: Vec<DeviceDto> = mgr
        .devices()
        .iter()
        .map(|d| DeviceDto {
            id: d.capabilities.id.clone(),
            name: d.capabilities.name.clone(),
            sample_rates_hz: d.capabilities.sample_rates_hz.clone(),
            max_output_channels: d.capabilities.max_output_channels,
            state: DeviceStateDto::from(d.state),
        })
        .collect();
    let has_available_device = mgr.has_available_device();
    Ok(Json(DevicesResponse {
        devices,
        has_available_device,
    }))
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{generate_refresh_token, hash_password, token_to_storage_key, JwtKeys},
        db::Db,
        middleware::jwt_auth,
        routes::devices::get_devices,
        security::validate_origin,
        state::AppState,
    };
    use axum::{middleware, routing::get, Router};
    use axum_test::TestServer;
    use control_protocol::Role;
    use control_server::ControlState;
    use device_manager::DeviceCapabilities;
    use serde_json::Value;
    use std::{
        fs,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };
    use topology::TopologyMode;

    fn test_keys() -> (Vec<u8>, Vec<u8>) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be valid")
            .as_nanos();
        let private_path = std::env::temp_dir().join(format!("open-iem-devices-test-{nonce}.pem"));
        let public_path =
            std::env::temp_dir().join(format!("open-iem-devices-test-{nonce}.pub.pem"));
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
            .route("/api/v1/devices", get(get_devices))
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

    #[tokio::test]
    async fn get_devices_requires_engineer_or_admin_role() {
        let (server, state) = build_test_app();
        let musician_token = seed_user_and_login(&state, "mus_devices_role", "pw", Role::Musician);
        server
            .get("/api/v1/devices")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(musician_token)
            .await
            .assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn get_devices_returns_empty_registry() {
        let (server, state) = build_test_app();
        let admin_token = seed_user_and_login(&state, "adm_devices_empty", "pw", Role::Admin);
        let resp = server
            .get("/api/v1/devices")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(admin_token)
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["devices"], serde_json::json!([]));
        assert_eq!(body["has_available_device"], false);
    }

    #[tokio::test]
    async fn get_devices_returns_snapshot_after_discover() {
        let (server, state) = build_test_app();

        // Pre-populate: discover one device then mark it failed so it enters
        // Recovering, then re-discover it so it enters Reconnected state.
        {
            let caps = DeviceCapabilities {
                id: "dev-001".to_owned(),
                name: "Test Headphone Amp".to_owned(),
                sample_rates_hz: vec![44_100, 48_000],
                max_output_channels: 2,
                supported_modes: vec![TopologyMode::ChannelMode],
            };
            let mut mgr = state.devices.lock().expect("device lock");
            mgr.discover(vec![caps.clone()])
                .expect("discover must succeed");
            mgr.mark_failed("dev-001")
                .expect("mark_failed must succeed");
            mgr.discover(vec![caps]).expect("re-discover must succeed");
            // State is now Reconnected (capabilities changed after failure).
        }

        let admin_token = seed_user_and_login(&state, "adm_devices_snapshot", "pw", Role::Admin);
        let resp = server
            .get("/api/v1/devices")
            .add_header("Origin", "http://localhost")
            .authorization_bearer(admin_token)
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        let devices = body["devices"].as_array().expect("devices must be array");
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0]["id"], "dev-001");
        assert_eq!(devices[0]["name"], "Test Headphone Amp");
        assert_eq!(devices[0]["state"], "reconnected");
        assert_eq!(body["has_available_device"], false);
    }
}
