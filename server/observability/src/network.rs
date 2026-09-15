//! Network quality metrics: late packets, reordered packets, jitter events.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::saturating_inc;

/// Atomic counters for network quality indicators.
#[derive(Debug, Default)]
pub struct NetworkMetrics {
    /// Packets that arrived after the playout deadline (late but not lost).
    late_packets: AtomicU64,
    /// Packets that arrived out of sequence order.
    reordered_packets: AtomicU64,
    /// Times the jitter estimator exceeded the configured threshold.
    jitter_events: AtomicU64,
}

/// Point-in-time snapshot of network quality counters.
#[derive(Clone, Debug, serde::Serialize)]
pub struct NetworkSnapshot {
    /// Total late-arriving packets since last reset.
    pub late_packets: u64,
    /// Total out-of-order packets since last reset.
    pub reordered_packets: u64,
    /// Total jitter threshold exceedances since last reset.
    pub jitter_events: u64,
}

impl NetworkMetrics {
    /// Record one packet that arrived after playout deadline.
    pub fn record_late(&self) {
        saturating_inc(&self.late_packets);
    }

    /// Record one out-of-order packet.
    pub fn record_reorder(&self) {
        saturating_inc(&self.reordered_packets);
    }

    /// Record one jitter threshold exceedance.
    pub fn record_jitter_event(&self) {
        saturating_inc(&self.jitter_events);
    }

    /// Reset all network counters.
    pub fn reset(&self) {
        self.late_packets.store(0, Ordering::Relaxed);
        self.reordered_packets.store(0, Ordering::Relaxed);
        self.jitter_events.store(0, Ordering::Relaxed);
    }

    /// Return a consistent snapshot.
    #[must_use]
    pub fn snapshot(&self) -> NetworkSnapshot {
        NetworkSnapshot {
            late_packets: self.late_packets.load(Ordering::Acquire),
            reordered_packets: self.reordered_packets.load(Ordering::Acquire),
            jitter_events: self.jitter_events.load(Ordering::Acquire),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_counters_are_independent() {
        let m = NetworkMetrics::default();
        m.record_late();
        m.record_late();
        m.record_reorder();
        m.record_jitter_event();
        let s = m.snapshot();
        assert_eq!(s.late_packets, 2);
        assert_eq!(s.reordered_packets, 1);
        assert_eq!(s.jitter_events, 1);
    }

    #[test]
    fn reset_clears_all() {
        let m = NetworkMetrics::default();
        m.record_late();
        m.record_reorder();
        m.record_jitter_event();
        m.reset();
        let s = m.snapshot();
        assert_eq!(s.late_packets, 0);
        assert_eq!(s.reordered_packets, 0);
        assert_eq!(s.jitter_events, 0);
    }
}
