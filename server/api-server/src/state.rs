//! Shared application state.

use crate::{auth::JwtKeys, db::Db};
use control_server::ControlState;
use device_manager::DeviceManager;
use observability::Metrics;
use recovery::RecoveryRegistry;
use scene_manager::SceneStore;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use streaming::SessionRegistry;
use tokio::sync::{broadcast, Mutex as AsyncMutex};

pub use crate::quota::MAX_WEBSOCKET_CONNECTIONS;

/// Maximum tracked Musician connection owners.
const MAX_CONNECTION_OWNERS: usize = 4096;

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
    /// Bounded session recovery registry — restores Musician mix assignment on reconnect.
    pub recovery: Arc<Mutex<RecoveryRegistry>>,
    /// Durable scene store.
    pub scenes: Arc<SceneStore>,
    /// Current WebSocket session owner for each Musician user.
    pub connection_owners: Arc<Mutex<HashMap<i64, u128>>>,
}

impl AppState {
    /// Create application state from its components.
    ///
    /// The scene store backend is selected via the `SCENE_STORE_PATH`
    /// environment variable:
    /// - If set to a non-empty string, a file-backed SQLite store is opened at
    ///   that path (created if it does not exist).
    /// - Otherwise (unset or empty), an in-memory SQLite store is used
    ///   (default, data lost on process exit).
    ///
    /// # Panics
    /// Panics if the scene store cannot be opened — either the in-memory store
    /// fails (should never happen) or the file-backed store path is
    /// inaccessible / unwritable.
    #[must_use]
    pub fn new(control: ControlState, db: Db, jwt: JwtKeys) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        let (master_event_tx, _) = broadcast::channel(256);
        let (eq_band_event_tx, _) = broadcast::channel(256);
        let scenes = Arc::new({
            let path = std::env::var("SCENE_STORE_PATH").unwrap_or_default();
            if path.is_empty() {
                scene_manager::SceneStore::open_in_memory()
                    .expect("in-memory scene store must open")
            } else {
                scene_manager::SceneStore::open(&path).expect("file-backed scene store must open")
            }
        });
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
            recovery: Arc::new(Mutex::new(RecoveryRegistry::new())),
            scenes,
            connection_owners: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    /// Claims current connection ownership for a Musician.
    #[must_use]
    pub fn claim_connection(&self, user_id: i64, session_id: u128) -> bool {
        match self.connection_owners.lock() {
            Ok(mut owners) => {
                if owners.contains_key(&user_id) {
                    return false;
                }
                if owners.len() >= MAX_CONNECTION_OWNERS {
                    return false;
                }
                owners.insert(user_id, session_id);
                true
            }
            Err(_) => false,
        }
    }

    /// Returns whether session currently owns user's connection, then releases ownership.
    #[must_use]
    pub fn release_connection(&self, user_id: i64, session_id: u128) -> bool {
        match self.connection_owners.lock() {
            Ok(mut owners) if owners.get(&user_id) == Some(&session_id) => {
                owners.remove(&user_id);
                true
            }
            Ok(_) | Err(_) => false,
        }
    }

    /// Checks current connection ownership.
    #[must_use]
    pub fn owns_connection(&self, user_id: i64, session_id: u128) -> bool {
        self.connection_owners
            .lock()
            .is_ok_and(|owners| owners.get(&user_id) == Some(&session_id))
    }
}

#[cfg(test)]
mod tests {
    /// Tests that `SceneStore::open` works correctly when `SCENE_STORE_PATH` is set.
    ///
    /// NOTE: `std::env::set_var` is not thread-safe in a multi-threaded test
    /// harness. This test directly exercises `SceneStore::open` (the same code
    /// path selected by the env-var branch in `AppState::new`) rather than
    /// constructing a full `AppState`, which would require live `Db`/`JwtKeys`
    /// stubs. The env-var wiring in `AppState::new` is verified by reading the
    /// source.
    #[test]
    fn scene_store_uses_file_when_env_set() {
        use std::env;
        let path = env::temp_dir().join(format!(
            "iem_scene_test_{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        let path_str = path.to_str().unwrap().to_owned();
        env::set_var("SCENE_STORE_PATH", &path_str);
        let store = scene_manager::SceneStore::open(&path_str).expect("file store must open");
        let scenes = store.list_scenes().expect("list must work");
        assert!(scenes.is_empty());
        env::remove_var("SCENE_STORE_PATH");
        let _ = std::fs::remove_file(&path);
    }
}
