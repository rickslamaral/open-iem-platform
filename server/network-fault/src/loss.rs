//! Fixed-rate packet-loss profile.
//!
//! Every `rate`-th packet (1-indexed) is dropped; all others are delivered
//! in original order.

use crate::{FaultError, Packet, SimResult};

/// Maximum allowed loss rate (denominator).  A rate of 1 would drop every
/// packet, leaving nothing to verify; cap at 2 (50 % loss).
pub const MAX_LOSS_RATE: usize = 1_000;

/// Fixed-rate loss injector.
#[derive(Debug, Clone)]
pub struct LossProfile {
    /// Drop every `rate`-th packet (e.g. 4 → 25 % loss).
    rate: usize,
}

impl LossProfile {
    /// Construct a loss profile.
    ///
    /// # Errors
    /// [`FaultError::InvalidParameter`] when `rate` is 0, 1, or > [`MAX_LOSS_RATE`].
    pub fn new(rate: usize) -> Result<Self, FaultError> {
        if rate < 2 {
            return Err(FaultError::InvalidParameter("rate must be ≥ 2"));
        }
        if rate > MAX_LOSS_RATE {
            return Err(FaultError::InvalidParameter("rate exceeds MAX_LOSS_RATE"));
        }
        Ok(Self { rate })
    }

    /// Apply the loss profile to `packets`.
    ///
    /// Packets at 1-indexed positions that are multiples of `rate` are dropped.
    #[must_use]
    pub fn apply(&self, packets: &[Packet]) -> SimResult {
        let mut delivered = Vec::with_capacity(packets.len());
        let mut dropped = 0usize;

        for (idx, pkt) in packets.iter().enumerate() {
            if (idx + 1) % self.rate == 0 {
                dropped += 1;
            } else {
                delivered.push(pkt.clone());
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
    fn rate_zero_rejected() {
        assert!(LossProfile::new(0).is_err());
    }

    #[test]
    fn rate_one_rejected() {
        assert!(LossProfile::new(1).is_err());
    }

    #[test]
    fn rate_above_max_rejected() {
        assert!(LossProfile::new(MAX_LOSS_RATE + 1).is_err());
    }

    #[test]
    fn rate_two_drops_half() {
        let profile = LossProfile::new(2).unwrap();
        let pkts = burst(10);
        let result = profile.apply(&pkts);
        assert_eq!(result.dropped, 5, "every 2nd packet dropped");
        assert_eq!(result.delivered.len(), 5);
        // Positions 1,3,5,7,9 (1-indexed) survive.
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        assert_eq!(seqs, vec![1, 3, 5, 7, 9]);
    }

    #[test]
    fn rate_four_drops_quarter() {
        let profile = LossProfile::new(4).unwrap();
        let pkts = burst(12);
        let result = profile.apply(&pkts);
        // Positions 4, 8, 12 dropped → 3 dropped.
        assert_eq!(result.dropped, 3);
        assert_eq!(result.delivered.len(), 9);
    }

    #[test]
    fn empty_input_returns_empty() {
        let profile = LossProfile::new(4).unwrap();
        let result = profile.apply(&[]);
        assert_eq!(result.dropped, 0);
        assert!(result.delivered.is_empty());
    }

    #[test]
    fn no_reorder_or_jitter_from_loss() {
        let profile = LossProfile::new(3).unwrap();
        let result = profile.apply(&burst(9));
        assert_eq!(result.reordered, 0);
        assert_eq!(result.jitter_events, 0);
    }

    #[test]
    fn delivered_packets_maintain_original_order() {
        let profile = LossProfile::new(3).unwrap();
        let pkts = burst(9);
        let result = profile.apply(&pkts);
        let seqs: Vec<u64> = result.delivered.iter().map(|p| p.sequence).collect();
        let mut sorted = seqs.clone();
        sorted.sort_unstable();
        assert_eq!(seqs, sorted, "delivered packets remain in original order");
    }
}
