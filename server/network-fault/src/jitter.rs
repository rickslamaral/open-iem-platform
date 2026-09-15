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
        let mut stream: Vec<Packet> = packets.to_vec();
        let mut jitter_events = 0usize;
        let mut reordered = 0usize;

        // Collect indices that will be delayed (0-indexed, interval-th = multiples of interval).
        let affected: Vec<usize> = (0..packets.len())
            .filter(|&i| (i + 1) % self.interval == 0)
            .collect();

        // Process in reverse so that earlier removals don't shift later indices.
        for &src_idx in affected.iter().rev() {
            let pkt = stream.remove(src_idx);
            let insert_at = (src_idx + self.delay_slots).min(stream.len());
            stream.insert(insert_at, pkt);
            jitter_events += 1;
            if insert_at != src_idx {
                reordered += 1;
            }
        }

        // Count how many packets are not at their original position.
        // The metric above already counts each moved packet once.

        SimResult {
            delivered: stream,
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
}
