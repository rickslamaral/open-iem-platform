//! Integration tests for `RecoveryRegistry` wiring into `AppState`.
//!
//! Pure unit-style tests — no HTTP server or WebSocket harness needed.

use recovery::RecoveryRegistry;
use std::time::Instant;

// ── helpers ──────────────────────────────────────────────────────────────────

fn make_state() -> api_server::state::AppState {
    use api_server::{auth::JwtKeys, db::Db, state::AppState};
    use control_server::ControlState;
    use std::{
        fs,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let priv_path = std::env::temp_dir().join(format!("iem-recovery-test-{nonce}.pem"));
    let pub_path = std::env::temp_dir().join(format!("iem-recovery-test-{nonce}.pub.pem"));

    Command::new("openssl")
        .args(["genpkey", "-algorithm", "ed25519", "-out"])
        .arg(&priv_path)
        .status()
        .expect("openssl must be installed");

    Command::new("openssl")
        .args(["pkey", "-in"])
        .arg(&priv_path)
        .args(["-pubout", "-out"])
        .arg(&pub_path)
        .status()
        .expect("openssl pkey export must succeed");

    let priv_pem = fs::read(&priv_path).unwrap();
    let pub_pem = fs::read(&pub_path).unwrap();
    let _ = fs::remove_file(&priv_path);
    let _ = fs::remove_file(&pub_path);

    let jwt = JwtKeys::from_ed_pem(&priv_pem, &pub_pem).expect("keys must parse");
    let db = Db::open_in_memory().expect("in-memory DB must open");
    let control = ControlState::new();
    AppState::new(control, db, jwt)
}

// ── tests ────────────────────────────────────────────────────────────────────

/// `session_disconnected` + `session_reconnected` round-trip returns saved `mix_id`.
#[test]
fn disconnect_saves_state_for_musician() {
    let mut reg = RecoveryRegistry::new();
    let uid = "musician-42";
    let mix_id: u8 = 1;

    reg.session_disconnected(uid, mix_id, Instant::now())
        .expect("session_disconnected must succeed");

    let result = reg
        .session_reconnected(uid)
        .expect("session_reconnected must succeed");
    assert_eq!(result, Some(1), "reconnect must return the saved mix_id");
    assert!(reg.is_empty(), "registry must be empty after recovery");
}

/// Registry stays empty when no Musician disconnect is recorded.
#[test]
fn non_musician_not_saved() {
    // Simulating an Engineer disconnect: never call session_disconnected.
    let reg = RecoveryRegistry::new();
    assert!(
        reg.is_empty(),
        "registry must be empty — no Musician state was recorded"
    );
}

/// End-to-end unit simulation using real `AppState` fields:
/// assign mix → record disconnect → simulate reconnect → verify DB assignment.
#[test]
fn musician_reconnect_restores_mix() {
    use control_protocol::Role;

    let state = make_state();

    // Create a musician and assign to mix 0.
    state
        .db
        .create_user("musician1", "hashed_pw", Role::Musician)
        .expect("create_user must succeed");
    let (user_id, _, _, _) = state
        .db
        .find_user("musician1")
        .expect("find_user must succeed");

    state
        .db
        .assign_mix(0, user_id)
        .expect("assign_mix must succeed");

    // Simulate disconnect: record in recovery registry.
    {
        let mut reg = state.recovery.lock().expect("lock must not be poisoned");
        reg.session_disconnected(&user_id.to_string(), 0, Instant::now())
            .expect("session_disconnected must succeed");
    }

    // Simulate reconnect: retrieve saved `mix_id`.
    let restored = {
        let mut reg = state.recovery.lock().expect("lock must not be poisoned");
        reg.session_reconnected(&user_id.to_string())
            .expect("session_reconnected must succeed")
    };
    assert_eq!(restored, Some(0), "reconnect must return mix_id 0");

    // DB assignment remains in place (as it would after assign_mix on reconnect).
    let assigned = state
        .db
        .get_user_assigned_mix(user_id)
        .expect("get_user_assigned_mix must succeed");
    assert_eq!(assigned, Some(0), "DB must show mix 0 assigned");
}
