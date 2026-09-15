//! Fixed-interval reorder profile.
//!
//! Every `interval`-th packet (1-indexed) is swapped with the following packet,
//! simulating brief out-of-order delivery common in Wi-Fi networks.

use crate::{FaultError, Packet, SimResult};

/// Maximum swap interval.
pub const MAX_INTERVAL: usize = 1_000;

/// Fixed-interval swap injector.
#[derive(Debug, Clone)]
pub struct ReorderProfile {
    /// Swap every `interval`-th packet with its successor (1-indexed, ≥ 2).
    interval: usize,
}

impl ReorderProfile {
    /// Construct a reorder profile.
    ///
    /// # Errors
    /// [`FaultError::InvalidParameter`] when `interval` is 0, 1, or > [`MAX_INTERVAL`].
    pub fn new(interval: usize) -> Result<Self, FaultError> {
        if interval < 2 {
            return Err(FaultError::InvalidParameter("interval must be ≥ 2"));
        }
        if interval > MAX_INTERVAL {
            return Err(FaultError::InvalidParameter(
                "interval exceeds MAX_INTERVAL",
            ));
        }
        Ok(Self { interval })
    }

    /// Apply the reorder profile to `packets`.
    ///
    /// At each `interval`-th position (0-indexed: `interval - 1`, `2*interval - 1`, …),
    /// if a successor exists, the two packets are swapped.
    #[must_use]
    pub fn apply(&self, packets: &[Packet]) -> SimResult {
        let mut stream: Vec<Packet> = packets.to_vec();
        let mut reordered = 0usize;
        let len = stream.len();

        let mut pos = self.interval - 1; // first 0-indexed swap position
        while pos + 1 < len {
            stream.swap(pos, pos + 1);
            reordered += 2; // both packets changed positions
            pos += self.interval;
        }

        SimResult {
            delivered: stream,
            dropped: 0,
            reordered,
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
    fn interval_zero_rejected() {
        assert!(ReorderProfile::new(0).is_err());
    }

    #[test]
    fn interval_one_rejected() {
        assert!(ReorderProfile::new(1).is_err());
    }

    #[test]
    fn above_max_rejected() {
        assert!(ReorderProfile::new(MAX_INTERVAL + 1).is_err());
    }

    #[test]
    fn no_packets_dropped() {
        let profile = ReorderProfile::new(3).unwrap();
        let pkts = burst(9);
        let result = profile.apply(&pkts);
        assert_eq!(result.dropped, 0);
        assert_eq!(result.delivered.len(), 9);
    }

    #[test]
    fn swap_at_interval_3() {
        // Positions 0-indexed: 2 and 5 get swapped with their successors.
        // Input:    seq 1,2,3,4,5,6,7,8,9
        // After swap at pos 2: 1,2,4,3,5,6,7,8,9
        // After swap at pos 5: 1,2,4,3,5,7,6,8,9
        let profile = ReorderProfile::new(3).unwrap();
        let result = profile.apply(&burst(9));
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        assert_eq!(seqs, vec![1, 2, 4, 3, 5, 7, 6, 8, 9]);
    }

    #[test]
    fn reordered_count_correct() {
        let profile = ReorderProfile::new(3).unwrap();
        // 9 packets → swaps at 0-indexed positions 2 and 5 → 2 swaps × 2 = 4.
        let result = profile.apply(&burst(9));
        assert_eq!(result.reordered, 4);
    }

    #[test]
    fn no_swap_when_last_has_no_successor() {
        // With interval=3 and 3 packets: swap at index 2, but no successor.
        let profile = ReorderProfile::new(3).unwrap();
        let result = profile.apply(&burst(3));
        // Index 2 has no successor → no swap.
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        assert_eq!(seqs, vec![1, 2, 3]);
        assert_eq!(result.reordered, 0);
    }

    #[test]
    fn empty_input_ok() {
        let profile = ReorderProfile::new(4).unwrap();
        let result = profile.apply(&[]);
        assert!(result.delivered.is_empty());
        assert_eq!(result.reordered, 0);
    }

    #[test]
    fn all_packets_delivered() {
        let profile = ReorderProfile::new(2).unwrap();
        let pkts = burst(10);
        let result = profile.apply(&pkts);
        let mut seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        seqs.sort_unstable();
        // All original sequences present.
        let expected: Vec<u64> = (1..=10).collect();
        assert_eq!(seqs, expected);
    }
}
