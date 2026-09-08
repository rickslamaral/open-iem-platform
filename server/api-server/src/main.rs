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
use control_server::ControlState;
use std::{env, fs};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

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
        .with_state(state)
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(middleware::from_fn(validate_origin))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Open IEM API server listening on {bind_addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
