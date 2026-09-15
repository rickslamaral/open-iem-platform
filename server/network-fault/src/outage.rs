//! Contiguous-window outage profile.
//!
//! Drops a fixed window of packets starting at a given sequence offset,
//! simulating a brief link outage.  Packets outside the window are delivered
//! in original order.

use crate::{FaultError, Packet, SimResult};

/// Maximum allowed outage window (in packets).
pub const MAX_WINDOW: usize = 256;

/// Contiguous outage injector.
#[derive(Debug, Clone)]
pub struct OutageProfile {
    /// 0-indexed start position in the input stream.
    start: usize,
    /// Number of consecutive packets to drop.
    window: usize,
}

impl OutageProfile {
    /// Construct an outage profile.
    ///
    /// # Errors
    /// [`FaultError::InvalidParameter`] when `window` is 0 or > [`MAX_WINDOW`].
    pub fn new(start: usize, window: usize) -> Result<Self, FaultError> {
        if window == 0 {
            return Err(FaultError::InvalidParameter("window must be ≥ 1"));
        }
        if window > MAX_WINDOW {
            return Err(FaultError::InvalidParameter("window exceeds MAX_WINDOW"));
        }
        Ok(Self { start, window })
    }

    /// Apply the outage to `packets`.
    ///
    /// Packets at indices `[start, start + window)` are dropped.  If `start`
    /// is beyond the stream end, no packets are dropped.
    #[must_use]
    pub fn apply(&self, packets: &[Packet]) -> SimResult {
        let end = self.start.saturating_add(self.window).min(packets.len());
        let actual_dropped = end.saturating_sub(self.start.min(packets.len()));

        let mut delivered = Vec::with_capacity(packets.len().saturating_sub(actual_dropped));
        for (idx, pkt) in packets.iter().enumerate() {
            if idx < self.start || idx >= end {
                delivered.push(pkt.clone());
            }
        }

        SimResult {
            delivered,
            dropped: actual_dropped,
            reordered: 0,
            jitter_events: 0,
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
    fn window_zero_rejected() {
        assert!(OutageProfile::new(0, 0).is_err());
    }

    #[test]
    fn window_above_max_rejected() {
        assert!(OutageProfile::new(0, MAX_WINDOW + 1).is_err());
    }

    #[test]
    fn drops_correct_window() {
        // Stream: seq 1..=10 (0-indexed 0..9). Outage at 3, window 3 → drop indices 3,4,5.
        let profile = OutageProfile::new(3, 3).unwrap();
        let result = profile.apply(&burst(10));
        assert_eq!(result.dropped, 3);
        assert_eq!(result.delivered.len(), 7);
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        assert_eq!(seqs, vec![1, 2, 3, 7, 8, 9, 10]);
    }

    #[test]
    fn outage_at_start() {
        let profile = OutageProfile::new(0, 5).unwrap();
        let result = profile.apply(&burst(10));
        assert_eq!(result.dropped, 5);
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        assert_eq!(seqs, vec![6, 7, 8, 9, 10]);
    }

    #[test]
    fn outage_at_end() {
        let profile = OutageProfile::new(7, 3).unwrap();
        let result = profile.apply(&burst(10));
        assert_eq!(result.dropped, 3);
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        assert_eq!(seqs, vec![1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn window_exceeds_stream_clamped() {
        // Start at index 8, window 10 → only indices 8,9 exist.
        let profile = OutageProfile::new(8, 10).unwrap();
        let result = profile.apply(&burst(10));
        assert_eq!(result.dropped, 2);
        assert_eq!(result.delivered.len(), 8);
    }

    #[test]
    fn start_beyond_stream_drops_nothing() {
        let profile = OutageProfile::new(20, 5).unwrap();
        let result = profile.apply(&burst(10));
        assert_eq!(result.dropped, 0);
        assert_eq!(result.delivered.len(), 10);
    }

    #[test]
    fn no_reorder_or_jitter_from_outage() {
        let profile = OutageProfile::new(2, 2).unwrap();
        let result = profile.apply(&burst(8));
        assert_eq!(result.reordered, 0);
        assert_eq!(result.jitter_events, 0);
    }

    #[test]
    fn empty_input_ok() {
        let profile = OutageProfile::new(0, 5).unwrap();
        let result = profile.apply(&[]);
        assert_eq!(result.dropped, 0);
        assert!(result.delivered.is_empty());
    }
}
