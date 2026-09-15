//! Reconnect fault profile.
//!
//! Simulates a client disconnect at a configurable split point, exercises the
//! [`recovery`] crate's [`RecoveryRegistry`] to restore mix assignment, and
//! resumes delivery of the remaining packets.
//!
//! This is an **L1 SIMULATED** profile: no network I/O occurs.

use crate::{FaultError, Packet};

/// Maximum byte length of a user ID used by this profile.
pub const MAX_USER_ID_BYTES: usize = 128;

/// Maximum mix ID value accepted by this profile.
pub const MAX_MIX_ID: u8 = 31;

/// Outcome of a reconnect simulation run.
#[derive(Debug, Clone)]
pub struct ReconnectResult {
    /// Packets delivered before the disconnect point.
    pub pre_disconnect: Vec<Packet>,
    /// Packets delivered after reconnect.
    pub post_reconnect: Vec<Packet>,
    /// Mix ID recovered for the user, or `None` if registry had no record.
    pub recovered_mix_id: Option<u8>,
    /// Total packets lost at the disconnect boundary (gap packets).
    pub lost_at_disconnect: usize,
}

/// Reconnect fault profile configuration.
#[derive(Debug, Clone)]
pub struct ReconnectProfile {
    /// 0-indexed position at which the disconnect occurs.
    split_at: usize,
    /// Number of in-flight packets lost at the disconnect boundary.
    gap: usize,
    /// User identity passed to the recovery registry.
    user_id: String,
    /// Mix assignment to store in the registry at disconnect.
    mix_id: u8,
}

impl ReconnectProfile {
    /// Construct a reconnect profile.
    ///
    /// # Errors
    /// [`FaultError::InvalidParameter`] for invalid `user_id` or `mix_id`.
    pub fn new(split_at: usize, gap: usize, user_id: &str, mix_id: u8) -> Result<Self, FaultError> {
        if user_id.is_empty() {
            return Err(FaultError::InvalidParameter("user_id must not be empty"));
        }
        if user_id.len() > MAX_USER_ID_BYTES {
            return Err(FaultError::InvalidParameter("user_id exceeds max length"));
        }
        if mix_id > MAX_MIX_ID {
            return Err(FaultError::InvalidParameter("mix_id exceeds MAX_MIX_ID"));
        }
        Ok(Self {
            split_at,
            gap,
            user_id: user_id.to_owned(),
            mix_id,
        })
    }

    /// Run the reconnect simulation on `packets`.
    ///
    /// 1. Delivers `packets[0..split_at]` as `pre_disconnect`.
    /// 2. Records disconnect in a fresh [`recovery::RecoveryRegistry`].
    /// 3. Skips `gap` packets (lost in-flight at boundary).
    /// 4. Attempts recovery to restore `mix_id`.
    /// 5. Delivers the remaining packets as `post_reconnect`.
    #[must_use]
    pub fn apply(&self, packets: &[Packet]) -> ReconnectResult {
        use recovery::RecoveryRegistry;
        use std::time::Instant;

        let split = self.split_at.min(packets.len());
        let pre_disconnect: Vec<Packet> = packets[..split].to_vec();

        // Simulate disconnect: record session in registry.
        let mut registry = RecoveryRegistry::new();
        let now = Instant::now();
        // Ignore error — mix_id is validated in `new`; user_id is valid.
        let _ = registry.session_disconnected(&self.user_id, self.mix_id, now);

        // Skip `gap` packets lost at the boundary.
        let resume_at = (split + self.gap).min(packets.len());
        let lost_at_disconnect = resume_at - split;

        // Attempt recovery.
        let recovered_mix_id = registry.session_reconnected(&self.user_id).ok().flatten();

        // Deliver remaining packets.
        let post_reconnect: Vec<Packet> = packets[resume_at..].to_vec();

        ReconnectResult {
            pre_disconnect,
            post_reconnect,
            recovered_mix_id,
            lost_at_disconnect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn burst(n: usize) -> Vec<Packet> {
        Packet::burst(1, n)
    }

    #[test]
    fn empty_user_id_rejected() {
        assert!(ReconnectProfile::new(5, 0, "", 0).is_err());
    }

    #[test]
    fn mix_id_too_large_rejected() {
        assert!(ReconnectProfile::new(5, 0, "alice", MAX_MIX_ID + 1).is_err());
    }

    #[test]
    fn user_id_too_long_rejected() {
        let long_id = "x".repeat(MAX_USER_ID_BYTES + 1);
        assert!(ReconnectProfile::new(5, 0, &long_id, 0).is_err());
    }

    #[test]
    fn mix_id_recovered_after_reconnect() {
        let profile = ReconnectProfile::new(5, 0, "alice", 7).unwrap();
        let result = profile.apply(&burst(10));
        assert_eq!(result.recovered_mix_id, Some(7), "mix ID must be recovered");
    }

    #[test]
    fn pre_and_post_split_correct() {
        let profile = ReconnectProfile::new(4, 0, "bob", 3).unwrap();
        let result = profile.apply(&burst(10));
        assert_eq!(result.pre_disconnect.len(), 4);
        assert_eq!(result.post_reconnect.len(), 6);
    }

    #[test]
    fn gap_removes_boundary_packets() {
        let profile = ReconnectProfile::new(4, 3, "carol", 1).unwrap();
        let result = profile.apply(&burst(10));
        // pre = 4, gap = 3, post = 3
        assert_eq!(result.pre_disconnect.len(), 4);
        assert_eq!(result.lost_at_disconnect, 3);
        assert_eq!(result.post_reconnect.len(), 3);
    }

    #[test]
    fn gap_beyond_stream_clamped() {
        let profile = ReconnectProfile::new(3, 100, "dave", 0).unwrap();
        let result = profile.apply(&burst(5));
        assert_eq!(result.pre_disconnect.len(), 3);
        assert_eq!(result.lost_at_disconnect, 2); // only 2 remain
        assert!(result.post_reconnect.is_empty());
    }

    #[test]
    fn split_beyond_stream_clamped() {
        let profile = ReconnectProfile::new(100, 0, "eve", 5).unwrap();
        let result = profile.apply(&burst(5));
        assert_eq!(result.pre_disconnect.len(), 5);
        assert!(result.post_reconnect.is_empty());
        // Recovery still works even with clamped split.
        assert_eq!(result.recovered_mix_id, Some(5));
    }

    #[test]
    fn zero_split_delivers_all_post() {
        let profile = ReconnectProfile::new(0, 0, "frank", 2).unwrap();
        let result = profile.apply(&burst(8));
        assert!(result.pre_disconnect.is_empty());
        assert_eq!(result.post_reconnect.len(), 8);
        assert_eq!(result.recovered_mix_id, Some(2));
    }

    #[test]
    fn empty_stream_ok() {
        let profile = ReconnectProfile::new(3, 2, "grace", 0).unwrap();
        let result = profile.apply(&[]);
        assert!(result.pre_disconnect.is_empty());
        assert!(result.post_reconnect.is_empty());
        assert_eq!(result.lost_at_disconnect, 0);
        // Registry still records and recovers.
        assert_eq!(result.recovered_mix_id, Some(0));
    }

    #[test]
    fn max_mix_id_accepted() {
        let profile = ReconnectProfile::new(2, 0, "henry", MAX_MIX_ID).unwrap();
        let result = profile.apply(&burst(4));
        assert_eq!(result.recovered_mix_id, Some(MAX_MIX_ID));
    }

    // ── Integration: reconnect result feeds observability counters ───────────

    #[test]
    fn lost_at_disconnect_feeds_observability() {
        use observability::{NetworkMetrics, ReceiverMetrics};

        let profile = ReconnectProfile::new(3, 5, "ivan", 1).unwrap();
        let result = profile.apply(&burst(15));

        let net = NetworkMetrics::default();
        let recv = ReceiverMetrics::default();

        // Record late-arrival events for packets that arrived after boundary.
        for _ in 0..result.lost_at_disconnect {
            net.record_late();
        }
        // Record dropped packets in receiver metrics.
        for _ in 0..result.lost_at_disconnect {
            recv.record_dropped();
        }

        let ns = net.snapshot();
        let rs = recv.snapshot();
        assert_eq!(ns.late_packets, 5);
        assert_eq!(rs.packets_dropped, 5);
    }
}
