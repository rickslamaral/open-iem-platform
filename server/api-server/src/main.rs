//! Binary entry point for api-server.
//!
//! Reads config from environment variables and starts the Axum HTTP server.
//!
//! Required env vars:
//!   `OPENIEM_JWT_PRIVATE_PEM` — path to Ed25519 private key PEM
//!   `OPENIEM_JWT_PUBLIC_PEM`  — path to Ed25519 public key PEM
//!   `OPENIEM_DB_PATH`         — path to SQLite database file
//!   `OPENIEM_BIND_ADDR`       — bind address (default: 127.0.0.1:8080); non-loopback requires explicit dev override

use api_server::{
    auth::JwtKeys,
    db::Db,
    middleware::jwt_auth,
    routes::{
        admin::{
            delete_user as admin_delete_user, list_sessions as admin_list_sessions,
            list_users as admin_list_users, revoke_session as admin_revoke_session,
        },
        audio::{ice_candidate, offer, sessions},
        auth::{create_user, login, logout, refresh},
        channels::{get_state, list_channels, set_channel_gain, set_channel_mute},
        config::{backup_config, restore_config},
        devices::get_devices,
        health::health,
        metrics::get_metrics,
        mixes::{
            assign_mix, get_send_state, list_mixes, set_send_gain, set_send_muted, set_send_pan,
            unassign_mix,
        },
        presets::list_presets,
        scenes::{
            backup_scenes, create_scene, delete_scene, get_active_scene, get_scene, list_scenes,
            recall_scene, restore_scenes, update_scene,
        },
        system::get_system_info,
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
use control_server::ControlState;
use std::{env, fs};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Detect legacy env var prefix (OPEN_IEM_*) from before Phase 22 and warn loudly.
    // These variables are silently ignored by the binary; failing to rename them means
    // secrets or bind addresses are not applied, which can produce insecure defaults.
    let legacy_vars = [
        "OPEN_IEM_BIND",
        "OPEN_IEM_DEV_ALLOW_NON_LOOPBACK",
        "OPEN_IEM_DB_PATH",
        "OPEN_IEM_JWT_PRIVATE_KEY_PATH",
        "OPEN_IEM_JWT_PUBLIC_KEY_PATH",
    ];
    for var in &legacy_vars {
        if env::var(var).is_ok() {
            tracing::warn!(
                "Detected legacy environment variable '{var}'. \
                 This variable is ignored — rename it to the OPENIEM_* equivalent. \
                 See server/api-server/src/main.rs for the correct names."
            );
        }
    }

    let private_pem_path = env::var("OPENIEM_JWT_PRIVATE_PEM")
        .unwrap_or_else(|_| "keys/ed25519_private.pem".to_owned());
    let public_pem_path =
        env::var("OPENIEM_JWT_PUBLIC_PEM").unwrap_or_else(|_| "keys/ed25519_public.pem".to_owned());
    let db_path = env::var("OPENIEM_DB_PATH").unwrap_or_else(|_| "openiem.db".to_owned());
    let bind_addr = env::var("OPENIEM_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
    let allow_insecure_http = env::var("OPENIEM_ALLOW_INSECURE_HTTP")
        .is_ok_and(|value| value.eq_ignore_ascii_case("true"));
    let parsed_bind_addr: std::net::SocketAddr = bind_addr
        .parse()
        .map_err(|_| anyhow::anyhow!("OPENIEM_BIND_ADDR must be a valid socket address"))?;
    if !parsed_bind_addr.ip().is_loopback() && !allow_insecure_http {
        anyhow::bail!(
            "refusing insecure HTTP on non-loopback address {bind_addr}; configure TLS reverse proxy or set OPENIEM_ALLOW_INSECURE_HTTP=true only for isolated development"
        );
    }

    let private_pem = fs::read(&private_pem_path)
        .unwrap_or_else(|_| panic!("cannot read OPENIEM_JWT_PRIVATE_PEM from {private_pem_path}"));
    let public_pem = fs::read(&public_pem_path)
        .unwrap_or_else(|_| panic!("cannot read OPENIEM_JWT_PUBLIC_PEM from {public_pem_path}"));

    let jwt = JwtKeys::from_ed_pem(&private_pem, &public_pem)?;
    let db = Db::open(&db_path)?;

    // Bootstrap soundtech user (idempotent — no-op if already exists).
    {
        const DEFAULT_PASSWORD: &str = "changeme-soundtech-2024";
        let soundtech_password = env::var("OPENIEM_SOUNDTECH_PASSWORD")
            .unwrap_or_else(|_| {
                tracing::warn!(
                    "OPENIEM_SOUNDTECH_PASSWORD not set — using compiled-in default.                      Change this password immediately in any non-development environment."
                );
                DEFAULT_PASSWORD.to_owned()
            });
        db.bootstrap_soundtech(&soundtech_password)
            .map_err(|e| anyhow::anyhow!("soundtech bootstrap failed: {e}"))?;
    }

    let state = AppState::new(ControlState::new(), db, jwt);

    let protected = Router::new()
        .route("/api/v1/state", get(get_state))
        .route("/api/v1/channels", get(list_channels))
        .route("/api/v1/telemetry", get(get_telemetry))
        .route("/api/v1/devices", get(get_devices))
        .route("/api/v1/scenes", get(list_scenes).post(create_scene))
        .route(
            "/api/v1/scenes/backup",
            get(backup_scenes).put(restore_scenes),
        )
        .route("/api/v1/scenes/active", get(get_active_scene))
        .route(
            "/api/v1/scenes/{id}",
            get(get_scene).put(update_scene).delete(delete_scene),
        )
        .route("/api/v1/scenes/{id}/recall", post(recall_scene))
        .route("/api/v1/metrics", get(get_metrics))
        .route("/api/v1/presets", get(list_presets))
        .route(
            "/api/v1/config/backup",
            get(backup_config).put(restore_config),
        )
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
        .route("/api/v1/system", get(get_system_info))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/refresh", post(refresh));

    let app = Router::new()
        .merge(protected)
        .merge(public)
        .with_state(state)
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(middleware::from_fn(validate_origin))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Open IEM API server listening on {bind_addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}
