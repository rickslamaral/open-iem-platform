//! Receiver packet metrics: received, dropped, and reconnect counters.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::saturating_inc;

/// Atomic counters for the receiver packet path.
#[derive(Debug, Default)]
pub struct ReceiverMetrics {
    /// Packets accepted by the jitter buffer from the transport.
    packets_received: AtomicU64,
    /// Packets discarded (arrived too late or duplicate).
    packets_dropped: AtomicU64,
    /// Number of receiver reconnect cycles completed.
    reconnect_count: AtomicU64,
}

/// Point-in-time snapshot of receiver counters.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ReceiverSnapshot {
    /// Total packets received since last reset.
    pub packets_received: u64,
    /// Total packets dropped since last reset.
    pub packets_dropped: u64,
    /// Total receiver reconnect cycles since last reset.
    pub reconnect_count: u64,
}

impl ReceiverMetrics {
    /// Record one packet accepted by the jitter buffer.
    pub fn record_received(&self) {
        saturating_inc(&self.packets_received);
    }

    /// Record one dropped packet (late or duplicate).
    pub fn record_dropped(&self) {
        saturating_inc(&self.packets_dropped);
    }

    /// Record one completed reconnect cycle.
    pub fn record_reconnect(&self) {
        saturating_inc(&self.reconnect_count);
    }

    /// Reset all receiver counters.
    pub fn reset(&self) {
        self.packets_received.store(0, Ordering::Relaxed);
        self.packets_dropped.store(0, Ordering::Relaxed);
        self.reconnect_count.store(0, Ordering::Relaxed);
    }

    /// Return a consistent snapshot.
    #[must_use]
    pub fn snapshot(&self) -> ReceiverSnapshot {
        ReceiverSnapshot {
            packets_received: self.packets_received.load(Ordering::Acquire),
            packets_dropped: self.packets_dropped.load(Ordering::Acquire),
            reconnect_count: self.reconnect_count.load(Ordering::Acquire),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_counters_are_independent() {
        let m = ReceiverMetrics::default();
        m.record_received();
        m.record_received();
        m.record_dropped();
        m.record_reconnect();
        let s = m.snapshot();
        assert_eq!(s.packets_received, 2);
        assert_eq!(s.packets_dropped, 1);
        assert_eq!(s.reconnect_count, 1);
    }

    #[test]
    fn reset_clears_all() {
        let m = ReceiverMetrics::default();
        m.record_received();
        m.record_dropped();
        m.record_reconnect();
        m.reset();
        let s = m.snapshot();
        assert_eq!(s.packets_received, 0);
        assert_eq!(s.packets_dropped, 0);
        assert_eq!(s.reconnect_count, 0);
    }
}
