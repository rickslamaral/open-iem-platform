//! Shared application state.

use crate::{auth::JwtKeys, db::Db};
use control_server::ControlState;
use std::sync::{Arc, Mutex};
use streaming::SessionRegistry;
use tokio::sync::Mutex as AsyncMutex;

/// Application state shared across Axum handlers.
#[derive(Clone)]
pub struct AppState {
    /// Mix engine control state (channels, mixes, revisions).
    pub control: Arc<Mutex<ControlState>>,
    /// SQLite user + refresh-token store.
    pub db: Db,
    /// JWT signing/verification keys.
    pub jwt: Arc<JwtKeys>,
    /// Serializes refresh issuance, preventing concurrent rotation races.
    pub refresh_lock: Arc<Mutex<()>>,
    /// WebRTC audio transport sessions.
    pub streaming: SessionRegistry,
    /// Serializes mix assignment changes with signaling ownership checks.
    pub mix_assignment_lock: Arc<AsyncMutex<()>>,
}

impl AppState {
    /// Create application state from its components.
    #[must_use]
    pub fn new(control: ControlState, db: Db, jwt: JwtKeys) -> Self {
        Self {
            control: Arc::new(Mutex::new(control)),
            db,
            jwt: Arc::new(jwt),
            refresh_lock: Arc::new(Mutex::new(())),
            streaming: SessionRegistry::new(),
            mix_assignment_lock: Arc::new(AsyncMutex::new(())),
        }
    }
}
