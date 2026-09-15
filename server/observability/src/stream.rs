//! Stream frame metrics: sent, lost, and PLC-recovered frame counters.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::saturating_inc;

/// Atomic counters for stream frame delivery.
#[derive(Debug, Default)]
pub struct StreamMetrics {
    /// Frames emitted by the media plane toward the transport.
    sent: AtomicU64,
    /// Frames declared lost (no packet received within jitter window).
    lost: AtomicU64,
    /// Frames recovered via Packet Loss Concealment (PLC).
    plc_recovered: AtomicU64,
}

/// Point-in-time snapshot of stream frame counters.
#[derive(Clone, Debug, serde::Serialize)]
pub struct StreamSnapshot {
    /// Total frames sent since last reset.
    pub frames_sent: u64,
    /// Total frames declared lost since last reset.
    pub frames_lost: u64,
    /// Total frames recovered via PLC since last reset.
    pub frames_plc_recovered: u64,
}

impl StreamMetrics {
    /// Record one frame sent by the media plane.
    pub fn record_sent(&self) {
        saturating_inc(&self.sent);
    }

    /// Record one lost frame (no packet received within jitter window).
    pub fn record_lost(&self) {
        saturating_inc(&self.lost);
    }

    /// Record one PLC-recovered frame.
    pub fn record_plc_recovered(&self) {
        saturating_inc(&self.plc_recovered);
    }

    /// Reset all stream counters.
    pub fn reset(&self) {
        self.sent.store(0, Ordering::Relaxed);
        self.lost.store(0, Ordering::Relaxed);
        self.plc_recovered.store(0, Ordering::Relaxed);
    }

    /// Return a consistent snapshot.
    #[must_use]
    pub fn snapshot(&self) -> StreamSnapshot {
        StreamSnapshot {
            frames_sent: self.sent.load(Ordering::Acquire),
            frames_lost: self.lost.load(Ordering::Acquire),
            frames_plc_recovered: self.plc_recovered.load(Ordering::Acquire),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_are_independent() {
        let m = StreamMetrics::default();
        m.record_sent();
        m.record_sent();
        m.record_lost();
        m.record_plc_recovered();
        let s = m.snapshot();
        assert_eq!(s.frames_sent, 2);
        assert_eq!(s.frames_lost, 1);
        assert_eq!(s.frames_plc_recovered, 1);
    }

    #[test]
    fn reset_clears_all() {
        let m = StreamMetrics::default();
        m.record_sent();
        m.record_lost();
        m.record_plc_recovered();
        m.reset();
        let s = m.snapshot();
        assert_eq!(s.frames_sent, 0);
        assert_eq!(s.frames_lost, 0);
        assert_eq!(s.frames_plc_recovered, 0);
    }
}
