//! Fixed-delay jitter profile.
//!
//! Every `interval`-th packet is held back by `delay_slots` positions relative
//! to a sliding window, simulating late-arriving packets.  Packets that arrive
//! out of their original position increment the jitter-event counter.

use crate::{FaultError, Packet, SimResult};

/// Maximum delay in packet slots.
pub const MAX_DELAY_SLOTS: usize = 32;
/// Maximum interval between jitter events.
pub const MAX_INTERVAL: usize = 1_000;

/// Fixed-delay jitter injector.
#[derive(Debug, Clone)]
pub struct JitterProfile {
    /// Inject delay every `interval`-th packet (1-indexed, ≥ 2).
    interval: usize,
    /// Number of positions to delay the affected packet.
    delay_slots: usize,
}

impl JitterProfile {
    /// Construct a jitter profile.
    ///
    /// # Errors
    /// [`FaultError::InvalidParameter`] for out-of-range parameters.
    pub fn new(interval: usize, delay_slots: usize) -> Result<Self, FaultError> {
        if interval < 2 {
            return Err(FaultError::InvalidParameter("interval must be ≥ 2"));
        }
        if interval > MAX_INTERVAL {
            return Err(FaultError::InvalidParameter(
                "interval exceeds MAX_INTERVAL",
            ));
        }
        if delay_slots == 0 {
            return Err(FaultError::InvalidParameter("delay_slots must be ≥ 1"));
        }
        if delay_slots > MAX_DELAY_SLOTS {
            return Err(FaultError::InvalidParameter(
                "delay_slots exceeds MAX_DELAY_SLOTS",
            ));
        }
        Ok(Self {
            interval,
            delay_slots,
        })
    }

    /// Apply jitter to `packets`.
    ///
    /// Every `interval`-th packet (1-indexed) is moved `delay_slots` positions
    /// forward in the delivery stream.  If the insertion index exceeds the
    /// stream length the packet is appended at the end.
    #[must_use]
    pub fn apply(&self, packets: &[Packet]) -> SimResult {
        // Assign each packet its intended delivery slot from the original stream,
        // then sort by slot. Stable original-index tie breaking keeps collisions
        // deterministic without index-shift bugs from repeated remove/insert.
        let mut scheduled: Vec<(usize, usize, Packet)> = packets
            .iter()
            .cloned()
            .enumerate()
            .map(|(original_index, packet)| {
                let target = if (original_index + 1) % self.interval == 0 {
                    original_index
                        .saturating_add(self.delay_slots)
                        .saturating_add(1)
                        .min(packets.len().saturating_sub(1))
                } else {
                    original_index
                };
                (target, original_index, packet)
            })
            .collect();
        scheduled.sort_by_key(|(target, original_index, _)| (*target, *original_index));

        let delivered: Vec<Packet> = scheduled
            .iter()
            .map(|(_, _, packet)| packet.clone())
            .collect();
        let reordered = scheduled
            .iter()
            .enumerate()
            .filter(|(delivery_index, (_, original_index, _))| delivery_index != original_index)
            .count();
        let jitter_events = packets
            .iter()
            .enumerate()
            .filter(|(index, _)| (index + 1) % self.interval == 0)
            .count();

        SimResult {
            delivered,
            dropped: 0,
            reordered,
            jitter_events,
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
        assert!(JitterProfile::new(0, 1).is_err());
    }

    #[test]
    fn interval_one_rejected() {
        assert!(JitterProfile::new(1, 1).is_err());
    }

    #[test]
    fn delay_zero_rejected() {
        assert!(JitterProfile::new(4, 0).is_err());
    }

    #[test]
    fn delay_above_max_rejected() {
        assert!(JitterProfile::new(4, MAX_DELAY_SLOTS + 1).is_err());
    }

    #[test]
    fn no_packets_lost_under_jitter() {
        let profile = JitterProfile::new(3, 2).unwrap();
        let pkts = burst(12);
        let result = profile.apply(&pkts);
        assert_eq!(result.dropped, 0, "jitter never drops packets");
        assert_eq!(result.delivered.len(), 12, "all 12 delivered");
    }

    #[test]
    fn jitter_event_counted() {
        let profile = JitterProfile::new(4, 1).unwrap();
        // 8 packets → positions 4 and 8 (1-indexed) are affected.
        let result = profile.apply(&burst(8));
        assert_eq!(result.jitter_events, 2);
    }

    #[test]
    fn late_packet_is_reordered() {
        // With delay_slots=2 the affected packet ends up after its successor.
        let profile = JitterProfile::new(2, 2).unwrap();
        let pkts = burst(6);
        let result = profile.apply(&pkts);
        // Some reordering must have occurred.
        assert!(result.reordered > 0, "packets should arrive out of order");
    }

    #[test]
    fn empty_input_ok() {
        let profile = JitterProfile::new(3, 1).unwrap();
        let result = profile.apply(&[]);
        assert!(result.delivered.is_empty());
        assert_eq!(result.jitter_events, 0);
    }

    #[test]
    fn single_packet_no_reorder() {
        let profile = JitterProfile::new(2, 2).unwrap();
        let result = profile.apply(&burst(1));
        assert_eq!(result.delivered.len(), 1);
        assert_eq!(result.reordered, 0);
    }

    #[test]
    fn multiple_delayed_packets_keep_deterministic_order() {
        let profile = JitterProfile::new(2, 2).unwrap();
        let result = profile.apply(&burst(6));
        let seqs: Vec<u64> = result
            .delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect();
        assert_eq!(seqs, vec![1, 3, 2, 5, 4, 6]);
        assert_eq!(result.reordered, 4);
    }
}
