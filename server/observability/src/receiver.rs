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
    /// Cumulative PLC frames generated across all gaps.
    plc_frames_total: AtomicU64,
    /// Peak consecutive PLC frames observed in a single gap.
    plc_consecutive_max: AtomicU64,
    /// Number of receiver failures that latched fail-safe mute.
    output_failures: AtomicU64,
    /// Packets discarded because they arrived after the expected sequence (late or duplicate).
    late_packets: AtomicU64,
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
    /// Cumulative PLC frames generated across all gaps since last reset.
    pub plc_frames_total: u64,
    /// Peak consecutive PLC frames observed in a single gap since last reset.
    pub plc_consecutive_max: u64,
    /// Failures that latched fail-safe mute.
    pub output_failures: u64,
    /// Packets discarded because they arrived late or as duplicates.
    pub late_packets: u64,
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

    /// Record one failure that latched fail-safe mute.
    pub fn record_output_failure(&self) {
        saturating_inc(&self.output_failures);
    }

    /// Record one late or duplicate packet discarded by the playout path.
    pub fn record_late(&self) {
        saturating_inc(&self.late_packets);
    }

    /// Record one PLC frame. `consecutive` is the current consecutive count
    /// (already incremented) for this gap. Lock-free, safe from near-RT paths.
    pub fn record_plc_frame(&self, consecutive: u32) {
        saturating_inc(&self.plc_frames_total);
        let c = u64::from(consecutive);
        let mut cur = self.plc_consecutive_max.load(Ordering::Relaxed);
        loop {
            if c <= cur {
                break;
            }
            match self.plc_consecutive_max.compare_exchange_weak(
                cur,
                c,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(v) => cur = v,
            }
        }
    }

    /// Reset all receiver counters.
    pub fn reset(&self) {
        self.packets_received.store(0, Ordering::Relaxed);
        self.packets_dropped.store(0, Ordering::Relaxed);
        self.reconnect_count.store(0, Ordering::Relaxed);
        self.plc_frames_total.store(0, Ordering::Relaxed);
        self.plc_consecutive_max.store(0, Ordering::Relaxed);
        self.output_failures.store(0, Ordering::Relaxed);
        self.late_packets.store(0, Ordering::Relaxed);
    }

    /// Return a best-effort snapshot.
    #[must_use]
    pub fn snapshot(&self) -> ReceiverSnapshot {
        ReceiverSnapshot {
            packets_received: self.packets_received.load(Ordering::Acquire),
            packets_dropped: self.packets_dropped.load(Ordering::Acquire),
            reconnect_count: self.reconnect_count.load(Ordering::Acquire),
            plc_frames_total: self.plc_frames_total.load(Ordering::Acquire),
            plc_consecutive_max: self.plc_consecutive_max.load(Ordering::Acquire),
            output_failures: self.output_failures.load(Ordering::Acquire),
            late_packets: self.late_packets.load(Ordering::Acquire),
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
        m.record_output_failure();
        m.record_late();
        m.reset();
        let s = m.snapshot();
        assert_eq!(s.packets_received, 0);
        assert_eq!(s.packets_dropped, 0);
        assert_eq!(s.reconnect_count, 0);
        assert_eq!(s.output_failures, 0);
        assert_eq!(s.late_packets, 0);
    }

    #[test]
    fn plc_frame_increments_total() {
        let m = ReceiverMetrics::default();
        m.record_plc_frame(1);
        m.record_plc_frame(2);
        assert_eq!(m.snapshot().plc_frames_total, 2);
    }

    #[test]
    fn plc_consecutive_max_tracks_peak() {
        let m = ReceiverMetrics::default();
        m.record_plc_frame(1);
        m.record_plc_frame(3);
        m.record_plc_frame(2);
        assert_eq!(m.snapshot().plc_consecutive_max, 3);
    }

    #[test]
    fn reset_clears_plc_counters() {
        let m = ReceiverMetrics::default();
        m.record_plc_frame(5);
        m.reset();
        let s = m.snapshot();
        assert_eq!(s.plc_frames_total, 0);
        assert_eq!(s.plc_consecutive_max, 0);
    }
}
