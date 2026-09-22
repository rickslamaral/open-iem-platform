//! Bandwidth-cap fault profile.
//!
//! Simulates a throttled link by partitioning the packet stream into fixed
//! windows of [`BandwidthProfile::window_size`] packets and enforcing a byte
//! budget of [`BandwidthProfile::max_bytes`] per window.  Packets that would
//! exceed the budget are dropped; earlier packets in the window are always
//! preferred.
//!
//! This is an L1-SIMULATED profile: no real network I/O, no timing.  The
//! "bandwidth" is expressed as *bytes allowed per N packets*, which gives a
//! deterministic, reproducible signal without a clock.

use crate::{FaultError, Packet, SimResult};

/// Maximum window size (packets per window).
pub const MAX_WINDOW_SIZE: usize = 1_000;

/// Maximum byte budget per window (1500 bytes × `MAX_WINDOW_SIZE`).
pub const MAX_BYTES_PER_WINDOW: usize = 1_500 * MAX_WINDOW_SIZE;

/// Bandwidth-cap injector.
///
/// Partitions the packet stream into windows of `window_size` packets.
/// Within each window, packets are admitted greedily until the byte budget
/// `max_bytes` is exhausted; remaining packets in that window are dropped.
#[derive(Debug, Clone)]
pub struct BandwidthProfile {
    /// Maximum payload bytes admitted per window.
    max_bytes: usize,
    /// Number of packets per window.
    window_size: usize,
}

impl BandwidthProfile {
    /// Construct a bandwidth-cap profile.
    ///
    /// # Errors
    ///
    /// [`FaultError::InvalidParameter`] when:
    /// - `max_bytes` is 0 or exceeds [`MAX_BYTES_PER_WINDOW`].
    /// - `window_size` is 0 or exceeds [`MAX_WINDOW_SIZE`].
    pub fn new(max_bytes: usize, window_size: usize) -> Result<Self, FaultError> {
        if max_bytes == 0 {
            return Err(FaultError::InvalidParameter("max_bytes must be ≥ 1"));
        }
        if max_bytes > MAX_BYTES_PER_WINDOW {
            return Err(FaultError::InvalidParameter(
                "max_bytes exceeds MAX_BYTES_PER_WINDOW",
            ));
        }
        if window_size == 0 {
            return Err(FaultError::InvalidParameter("window_size must be ≥ 1"));
        }
        if window_size > MAX_WINDOW_SIZE {
            return Err(FaultError::InvalidParameter(
                "window_size exceeds MAX_WINDOW_SIZE",
            ));
        }
        Ok(Self {
            max_bytes,
            window_size,
        })
    }

    /// Apply the bandwidth cap to `packets`.
    ///
    /// Packets are processed in chunks of `window_size`.  Within each chunk,
    /// packets are admitted greedily until the byte budget is exhausted; any
    /// remaining packets in that chunk are dropped.
    #[must_use]
    pub fn apply(&self, packets: &[Packet]) -> SimResult {
        let mut delivered = Vec::with_capacity(packets.len());
        let mut dropped = 0usize;

        for chunk in packets.chunks(self.window_size) {
            let mut budget = self.max_bytes;
            for pkt in chunk {
                if pkt.payload.len() <= budget {
                    budget -= pkt.payload.len();
                    delivered.push(pkt.clone());
                } else {
                    dropped += 1;
                }
            }
        }

        SimResult {
            delivered,
            dropped,
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
    fn max_bytes_zero_rejected() {
        assert!(BandwidthProfile::new(0, 4).is_err());
    }

    #[test]
    fn window_size_zero_rejected() {
        assert!(BandwidthProfile::new(100, 0).is_err());
    }

    #[test]
    fn max_bytes_above_limit_rejected() {
        assert!(BandwidthProfile::new(MAX_BYTES_PER_WINDOW + 1, 4).is_err());
    }

    #[test]
    fn window_size_above_limit_rejected() {
        assert!(BandwidthProfile::new(100, MAX_WINDOW_SIZE + 1).is_err());
    }

    #[test]
    fn generous_budget_passes_all() {
        // Each sentinel packet has a 1-byte payload.  Budget of 100 in a
        // window of 10 → all 10 packets fit.
        let profile = BandwidthProfile::new(100, 10).unwrap();
        let pkts = burst(10);
        let result = profile.apply(&pkts);
        assert_eq!(result.dropped, 0);
        assert_eq!(result.delivered.len(), 10);
    }

    #[test]
    fn tight_budget_drops_overflow() {
        // Sentinel payload = 1 byte.  Budget of 3 in a window of 5 →
        // first 3 admitted, last 2 dropped per window.
        let profile = BandwidthProfile::new(3, 5).unwrap();
        let pkts = burst(5);
        let result = profile.apply(&pkts);
        assert_eq!(result.delivered.len(), 3);
        assert_eq!(result.dropped, 2);
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        assert_eq!(seqs, vec![1, 2, 3]);
    }

    #[test]
    fn budget_resets_per_window() {
        // Window of 3, budget of 2. Two windows of 3 packets each.
        // Window 1: seq 1,2 admitted, seq 3 dropped.
        // Window 2: seq 4,5 admitted, seq 6 dropped.
        let profile = BandwidthProfile::new(2, 3).unwrap();
        let pkts = burst(6);
        let result = profile.apply(&pkts);
        assert_eq!(result.dropped, 2);
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        assert_eq!(seqs, vec![1, 2, 4, 5]);
    }

    #[test]
    fn empty_input_returns_empty() {
        let profile = BandwidthProfile::new(100, 10).unwrap();
        let result = profile.apply(&[]);
        assert_eq!(result.dropped, 0);
        assert!(result.delivered.is_empty());
    }

    #[test]
    fn partial_last_window() {
        // 7 packets, window 4, budget 3.
        // Window 1 (pkts 1-4): seq 1,2,3 admitted, seq 4 dropped.
        // Window 2 (pkts 5-7, only 3): seq 5,6,7 all fit under budget 3.
        let profile = BandwidthProfile::new(3, 4).unwrap();
        let pkts = burst(7);
        let result = profile.apply(&pkts);
        assert_eq!(result.delivered.len(), 6);
        assert_eq!(result.dropped, 1);
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        assert_eq!(seqs, vec![1, 2, 3, 5, 6, 7]);
    }

    #[test]
    fn zero_budget_effect_via_minimum_budget() {
        // Budget of 1 with 1-byte sentinel payloads: every window admits exactly one packet.
        let profile = BandwidthProfile::new(1, 4).unwrap();
        let pkts = burst(8);
        let result = profile.apply(&pkts);
        // Window 1: seq 1 admitted, seq 2-4 dropped (budget 1 after seq 1).
        // Window 2: seq 5 admitted, seq 6-8 dropped.
        assert_eq!(result.delivered.len(), 2);
        assert_eq!(result.dropped, 6);
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        assert_eq!(seqs, vec![1, 5]);
    }

    #[test]
    fn variable_payload_sizes_consume_byte_budget_greedily() {
        let profile = BandwidthProfile::new(7, 4).unwrap();
        let packets = vec![
            Packet {
                sequence: 1,
                payload: vec![0; 3],
            },
            Packet {
                sequence: 2,
                payload: vec![0; 4],
            },
            Packet {
                sequence: 3,
                payload: vec![0; 1],
            },
            Packet {
                sequence: 4,
                payload: vec![0; 2],
            },
        ];

        let result = profile.apply(&packets);

        assert_eq!(result.dropped, 2);
        assert_eq!(
            result
                .delivered
                .iter()
                .map(|packet| packet.sequence)
                .collect::<Vec<_>>(),
            vec![1, 2],
            "budget must account for payload bytes, not packet count"
        );
    }

    #[test]
    fn no_reorder_or_jitter_from_bandwidth() {
        let profile = BandwidthProfile::new(50, 10).unwrap();
        let result = profile.apply(&burst(10));
        assert_eq!(result.reordered, 0);
        assert_eq!(result.jitter_events, 0);
    }

    #[test]
    fn delivered_order_preserved() {
        let profile = BandwidthProfile::new(5, 10).unwrap();
        let pkts = burst(10);
        let result = profile.apply(&pkts);
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        let mut sorted = seqs.clone();
        sorted.sort_unstable();
        assert_eq!(seqs, sorted, "delivery order must be preserved");
    }
}
