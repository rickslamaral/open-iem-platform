//! Shared application state.

use crate::{auth::JwtKeys, db::Db};
use control_server::ControlState;
use std::sync::{Arc, Mutex};
use streaming::SessionRegistry;
use tokio::sync::{broadcast, Mutex as AsyncMutex, Semaphore};

/// Maximum number of concurrently upgraded WebSocket connections per process.
pub const MAX_WEBSOCKET_CONNECTIONS: usize = 64;

/// State-delta event broadcast to all connected WebSocket sessions after a
/// send mutation (gain / pan / mute).  Each session filters by role and
/// mix ownership before forwarding to the client.
#[derive(Clone, Debug)]
pub struct SendDelta {
    /// Mix slot index that changed.
    pub mix_index: u8,
    /// Channel slot index within the mix.
    pub channel_index: u8,
    /// Current gain in dBFS after mutation.
    pub gain_db: f32,
    /// Current pan after mutation (−1.0 left … 1.0 right).
    pub pan: f32,
    /// Current mute state after mutation.
    pub muted: bool,
    /// Monotonic state revision after mutation.
    pub revision: u64,
    /// Unique WebSocket session that originated this mutation.
    pub originator_session_id: u128,
}

/// Master-level mutation broadcast (gain / mute on the mix bus).
/// Each session filters by role and mix ownership before forwarding.
#[derive(Clone, Debug)]
pub struct MasterDelta {
    /// Mix slot index that changed.
    pub mix_index: u8,
    /// Current master gain in dBFS after mutation.
    pub master_gain_db: f32,
    /// Current master mute state after mutation.
    pub master_muted: bool,
    /// Monotonic mix revision after mutation.
    pub revision: u64,
    /// Unique WebSocket session that originated this mutation.
    pub originator_session_id: u128,
}

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
    /// Broadcast channel for send mutations.  All WS sessions subscribe and
    /// filter events by role / assigned mix before forwarding to the client.
    pub event_tx: broadcast::Sender<SendDelta>,
    /// Broadcast channel for master mutations (gain / mute on the mix bus).
    pub master_event_tx: broadcast::Sender<MasterDelta>,
    /// Global cap on upgraded WebSocket connections.
    pub websocket_connections: Arc<Semaphore>,
}

impl AppState {
    /// Create application state from its components.
    #[must_use]
    pub fn new(control: ControlState, db: Db, jwt: JwtKeys) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        let (master_event_tx, _) = broadcast::channel(256);
        Self {
            control: Arc::new(Mutex::new(control)),
            db,
            jwt: Arc::new(jwt),
            refresh_lock: Arc::new(Mutex::new(())),
            streaming: SessionRegistry::new(),
            mix_assignment_lock: Arc::new(AsyncMutex::new(())),
            event_tx,
            master_event_tx,
            websocket_connections: Arc::new(Semaphore::new(MAX_WEBSOCKET_CONNECTIONS)),
        }
    }
}
