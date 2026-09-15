//! Bounded session recovery state machine for the Open IEM Platform.
//!
//! Tracks disconnected sessions so that reconnecting clients can have their
//! mix assignment restored without any state leaking between sessions.
//!
//! # Guarantees
//! - All bounds are enforced at call time; no method ever panics on out-of-range input.
//! - One session's state never affects another's (full isolation).
//! - No I/O, no async, no locking — pure in-memory state machine.

#![deny(missing_docs, unsafe_code)]

use std::collections::HashMap;
use std::time::Instant;

use thiserror::Error;

/// Maximum number of disconnected sessions the registry tracks concurrently.
pub const MAX_SESSIONS: usize = 64;

/// Maximum byte length of a user ID accepted by the registry.
pub const MAX_USER_ID_BYTES: usize = 128;

/// Inclusive maximum mix ID value accepted by the registry.
pub const MAX_MIX_ID: u8 = 31;

/// Errors returned by [`RecoveryRegistry`] operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RecoveryError {
    /// Registry is at capacity ([`MAX_SESSIONS`] entries already stored).
    #[error("recovery registry is at capacity ({MAX_SESSIONS} sessions)")]
    TooManySessions,

    /// The provided user ID exceeds [`MAX_USER_ID_BYTES`] bytes.
    #[error("user_id exceeds {MAX_USER_ID_BYTES} bytes")]
    UserIdTooLong,

    /// The provided mix ID exceeds [`MAX_MIX_ID`].
    #[error("mix_id exceeds MAX_MIX_ID ({MAX_MIX_ID})")]
    InvalidMixId,

    /// The provided user ID is an empty string.
    #[error("user_id must not be empty")]
    EmptyUserId,
}

/// Internal per-session record.
#[derive(Debug, Clone)]
struct SessionRecord {
    mix_id: u8,
    disconnected_at: Instant,
}

/// Bounded registry of disconnected session state.
///
/// Stores `(user_id → mix_id, disconnect_instant)` for up to [`MAX_SESSIONS`]
/// sessions simultaneously. All bounds are validated at call time.
#[derive(Debug, Default)]
pub struct RecoveryRegistry {
    entries: HashMap<String, SessionRecord>,
}

impl RecoveryRegistry {
    /// Creates a new, empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a session as disconnected with its current mix assignment.
    ///
    /// If a record for `user_id` already exists it is **overwritten** (no
    /// duplicate entry is created). Capacity is only checked when adding a
    /// brand-new entry.
    ///
    /// # Errors
    /// - [`RecoveryError::EmptyUserId`] — `user_id` is empty.
    /// - [`RecoveryError::UserIdTooLong`] — `user_id` exceeds [`MAX_USER_ID_BYTES`] bytes.
    /// - [`RecoveryError::InvalidMixId`] — `mix_id` exceeds [`MAX_MIX_ID`].
    /// - [`RecoveryError::TooManySessions`] — registry is full and `user_id` has no existing entry.
    pub fn session_disconnected(
        &mut self,
        user_id: &str,
        mix_id: u8,
        now: Instant,
    ) -> Result<(), RecoveryError> {
        validate_user_id(user_id)?;
        if mix_id > MAX_MIX_ID {
            return Err(RecoveryError::InvalidMixId);
        }

        // If entry already exists, overwrite without capacity check.
        if self.entries.contains_key(user_id) {
            self.entries.insert(
                user_id.to_owned(),
                SessionRecord {
                    mix_id,
                    disconnected_at: now,
                },
            );
            return Ok(());
        }

        // New entry — check capacity first.
        if self.entries.len() >= MAX_SESSIONS {
            return Err(RecoveryError::TooManySessions);
        }

        self.entries.insert(
            user_id.to_owned(),
            SessionRecord {
                mix_id,
                disconnected_at: now,
            },
        );
        Ok(())
    }

    /// Attempts to recover a disconnected session.
    ///
    /// Returns `Ok(Some(mix_id))` and removes the entry if one exists for
    /// `user_id`, or `Ok(None)` if no record is found.
    ///
    /// # Errors
    /// - [`RecoveryError::EmptyUserId`] — `user_id` is empty.
    /// - [`RecoveryError::UserIdTooLong`] — `user_id` exceeds [`MAX_USER_ID_BYTES`] bytes.
    pub fn session_reconnected(&mut self, user_id: &str) -> Result<Option<u8>, RecoveryError> {
        validate_user_id(user_id)?;
        Ok(self.entries.remove(user_id).map(|r| r.mix_id))
    }

    /// Removes all entries whose `disconnected_at` instant is **strictly before**
    /// `deadline`.
    ///
    /// Returns the number of entries purged.
    pub fn purge_expired(&mut self, deadline: Instant) -> usize {
        let before = self.entries.len();
        self.entries.retain(|_, r| r.disconnected_at >= deadline);
        before - self.entries.len()
    }

    /// Returns the number of disconnected sessions currently tracked.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when no sessions are tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Validates `user_id` length and non-emptiness.
fn validate_user_id(user_id: &str) -> Result<(), RecoveryError> {
    if user_id.is_empty() {
        return Err(RecoveryError::EmptyUserId);
    }
    if user_id.len() > MAX_USER_ID_BYTES {
        return Err(RecoveryError::UserIdTooLong);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn now() -> Instant {
        Instant::now()
    }

    // ── basic record + recover ───────────────────────────────────────────────

    #[test]
    fn basic_record_and_recover() {
        let mut reg = RecoveryRegistry::new();
        let t = now();
        reg.session_disconnected("alice", 5, t).unwrap();
        let mix = reg.session_reconnected("alice").unwrap();
        assert_eq!(mix, Some(5));
        assert!(reg.is_empty());
    }

    // ── reconnect when no record returns None ────────────────────────────────

    #[test]
    fn reconnect_no_record_returns_none() {
        let mut reg = RecoveryRegistry::new();
        let result = reg.session_reconnected("ghost").unwrap();
        assert_eq!(result, None);
    }

    // ── capacity limit ───────────────────────────────────────────────────────

    #[test]
    fn capacity_limit() {
        let mut reg = RecoveryRegistry::new();
        let t = now();
        for i in 0..MAX_SESSIONS {
            let id = format!("user_{i}");
            reg.session_disconnected(&id, 0, t).unwrap();
        }
        assert_eq!(reg.len(), MAX_SESSIONS);
        let err = reg.session_disconnected("overflow", 0, t).unwrap_err();
        assert_eq!(err, RecoveryError::TooManySessions);
    }

    // ── double-disconnect overwrites, no duplicate ───────────────────────────

    #[test]
    fn double_disconnect_overwrites() {
        let mut reg = RecoveryRegistry::new();
        let t = now();
        reg.session_disconnected("bob", 1, t).unwrap();
        reg.session_disconnected("bob", 7, t).unwrap(); // overwrite
        assert_eq!(reg.len(), 1, "no duplicate entry created");
        let mix = reg.session_reconnected("bob").unwrap();
        assert_eq!(mix, Some(7), "latest mix_id returned");
    }

    // ── expiry purge ─────────────────────────────────────────────────────────

    #[test]
    fn expiry_purge() {
        let mut reg = RecoveryRegistry::new();
        let early = now();
        reg.session_disconnected("old_user", 2, early).unwrap();

        // Small sleep so `later` is definitely after `early`.
        std::thread::sleep(Duration::from_millis(5));
        let later = Instant::now();

        reg.session_disconnected("new_user", 3, later).unwrap();

        // Purge entries with disconnected_at < later
        let purged = reg.purge_expired(later);
        assert_eq!(purged, 1, "one old entry purged");
        assert_eq!(reg.len(), 1, "new_user survives");
        assert!(reg.session_reconnected("new_user").unwrap().is_some());
    }

    // ── purge count accurate ─────────────────────────────────────────────────

    #[test]
    fn purge_count_accurate() {
        let mut reg = RecoveryRegistry::new();
        let t = now();
        for i in 0..5_u8 {
            reg.session_disconnected(&format!("u{i}"), i, t).unwrap();
        }
        std::thread::sleep(Duration::from_millis(5));
        let deadline = Instant::now();
        let purged = reg.purge_expired(deadline);
        assert_eq!(purged, 5);
        assert!(reg.is_empty());
    }

    // ── isolation between sessions ───────────────────────────────────────────

    #[test]
    fn isolation_between_sessions() {
        let mut reg = RecoveryRegistry::new();
        let t = now();
        reg.session_disconnected("carol", 10, t).unwrap();
        reg.session_disconnected("dave", 20, t).unwrap();

        // Recovering carol must not affect dave.
        assert_eq!(reg.session_reconnected("carol").unwrap(), Some(10));
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.session_reconnected("dave").unwrap(), Some(20));
        assert!(reg.is_empty());
    }

    // ── oversized user_id ────────────────────────────────────────────────────

    #[test]
    fn oversized_user_id_rejected() {
        let mut reg = RecoveryRegistry::new();
        let long_id = "x".repeat(MAX_USER_ID_BYTES + 1);
        let err = reg.session_disconnected(&long_id, 0, now()).unwrap_err();
        assert_eq!(err, RecoveryError::UserIdTooLong);
    }

    #[test]
    fn max_length_user_id_accepted() {
        let mut reg = RecoveryRegistry::new();
        let id = "x".repeat(MAX_USER_ID_BYTES);
        reg.session_disconnected(&id, 0, now()).unwrap();
        assert_eq!(reg.len(), 1);
    }

    // ── invalid mix_id ───────────────────────────────────────────────────────

    #[test]
    fn invalid_mix_id_rejected() {
        let mut reg = RecoveryRegistry::new();
        let err = reg
            .session_disconnected("eve", MAX_MIX_ID + 1, now())
            .unwrap_err();
        assert_eq!(err, RecoveryError::InvalidMixId);
    }

    #[test]
    fn max_mix_id_accepted() {
        let mut reg = RecoveryRegistry::new();
        reg.session_disconnected("frank", MAX_MIX_ID, now())
            .unwrap();
        assert_eq!(reg.session_reconnected("frank").unwrap(), Some(MAX_MIX_ID));
    }

    // ── empty user_id ────────────────────────────────────────────────────────

    #[test]
    fn empty_user_id_rejected_on_disconnect() {
        let mut reg = RecoveryRegistry::new();
        let err = reg.session_disconnected("", 0, now()).unwrap_err();
        assert_eq!(err, RecoveryError::EmptyUserId);
    }

    #[test]
    fn empty_user_id_rejected_on_reconnect() {
        let mut reg = RecoveryRegistry::new();
        let err = reg.session_reconnected("").unwrap_err();
        assert_eq!(err, RecoveryError::EmptyUserId);
    }

    // ── double-disconnect at capacity does not block ─────────────────────────

    #[test]
    fn double_disconnect_at_capacity_allowed() {
        let mut reg = RecoveryRegistry::new();
        let t = now();
        // Fill to capacity.
        for i in 0..MAX_SESSIONS {
            reg.session_disconnected(&format!("u{i}"), 0, t).unwrap();
        }
        // Overwriting an existing entry must succeed even at capacity.
        reg.session_disconnected("u0", 5, t).unwrap();
        assert_eq!(reg.len(), MAX_SESSIONS);
    }
}
