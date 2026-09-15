//! Device lifecycle metrics: loss and recovery event counters.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::saturating_inc;

/// Atomic counters for device lifecycle events.
#[derive(Debug, Default)]
pub struct DeviceMetrics {
    /// Number of times a device entered the Recovering state.
    loss_events: AtomicU64,
    /// Number of times a device transitioned from Reconnected → Available.
    recovery_events: AtomicU64,
}

/// Point-in-time snapshot of device lifecycle counters.
#[derive(Clone, Debug, serde::Serialize)]
pub struct DeviceSnapshot {
    /// Total device loss events since last reset.
    pub loss_events: u64,
    /// Total device recovery events since last reset.
    pub recovery_events: u64,
}

impl DeviceMetrics {
    /// Record a device loss event (device disappeared or failed).
    pub fn record_loss(&self) {
        saturating_inc(&self.loss_events);
    }

    /// Record a successful device recovery (Available after rediscovery).
    pub fn record_recovery(&self) {
        saturating_inc(&self.recovery_events);
    }

    /// Reset counters to zero.
    pub fn reset(&self) {
        self.loss_events.store(0, Ordering::Relaxed);
        self.recovery_events.store(0, Ordering::Relaxed);
    }

    /// Return a consistent snapshot.
    #[must_use]
    pub fn snapshot(&self) -> DeviceSnapshot {
        DeviceSnapshot {
            loss_events: self.loss_events.load(Ordering::Acquire),
            recovery_events: self.recovery_events.load(Ordering::Acquire),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loss_and_recovery_increment_independently() {
        let m = DeviceMetrics::default();
        m.record_loss();
        m.record_loss();
        m.record_recovery();
        let s = m.snapshot();
        assert_eq!(s.loss_events, 2);
        assert_eq!(s.recovery_events, 1);
    }

    #[test]
    fn reset_clears_counters() {
        let m = DeviceMetrics::default();
        m.record_loss();
        m.record_recovery();
        m.reset();
        let s = m.snapshot();
        assert_eq!(s.loss_events, 0);
        assert_eq!(s.recovery_events, 0);
    }
}
