//! Shared application state.

use crate::{auth::JwtKeys, db::Db};
use control_server::ControlState;
use device_manager::DeviceManager;
use observability::Metrics;
use std::sync::{Arc, Mutex};
use streaming::SessionRegistry;
use tokio::sync::{broadcast, Mutex as AsyncMutex};

pub use crate::quota::MAX_WEBSOCKET_CONNECTIONS;

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

/// EQ band mutation broadcast.
/// All Engineer/Admin sessions receive this; Musician sessions do NOT
/// (they cannot configure EQ, so they do not need to track it).
#[derive(Clone, Debug)]
pub struct EqBandDelta {
    /// Mix slot index that changed.
    pub mix_index: u8,
    /// EQ band index.
    pub band_index: u8,
    /// Centre frequency in Hz.
    pub frequency_hz: f32,
    /// Gain in dB.
    pub gain_db: f32,
    /// Q factor.
    pub q: f32,
    /// Whether the band is active.
    pub enabled: bool,
    /// Monotonic EQ revision after mutation.
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
    /// Broadcast channel for EQ band mutations (Engineer/Admin only).
    pub eq_band_event_tx: broadcast::Sender<EqBandDelta>,
    /// Shared quotas for upgraded WebSocket connections.
    pub websocket_connections: crate::quota::WebSocketQuota,
    /// Bounded failed-authentication limiter for WebSocket upgrades.
    pub websocket_auth_failures: crate::quota::WebSocketAuthFailureLimiter,
    /// Observability metrics counters.
    pub metrics: Arc<Metrics>,
    /// Bounded audio device registry.
    pub devices: Arc<Mutex<DeviceManager>>,
}

impl AppState {
    /// Create application state from its components.
    #[must_use]
    pub fn new(control: ControlState, db: Db, jwt: JwtKeys) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        let (master_event_tx, _) = broadcast::channel(256);
        let (eq_band_event_tx, _) = broadcast::channel(256);
        Self {
            control: Arc::new(Mutex::new(control)),
            db,
            jwt: Arc::new(jwt),
            refresh_lock: Arc::new(Mutex::new(())),
            streaming: SessionRegistry::new(),
            mix_assignment_lock: Arc::new(AsyncMutex::new(())),
            event_tx,
            master_event_tx,
            eq_band_event_tx,
            websocket_connections: crate::quota::WebSocketQuota::default(),
            websocket_auth_failures: crate::quota::WebSocketAuthFailureLimiter::default(),
            metrics: Arc::new(Metrics::new()),
            devices: Arc::new(Mutex::new(DeviceManager::new())),
        }
    }
}
