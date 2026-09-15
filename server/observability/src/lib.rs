//! Open IEM Platform — bounded audio observability metrics.
//!
//! All counters use [`AtomicU64`] and are updated from any thread without
//! locks. Snapshots are point-in-time reads; values may advance between calls.
//!
//! Modules:
//! - [`audio`] — XRUN counter and frames-processed counter for the audio backend.
//! - [`device`] — Device availability event counters (loss / recovery events).
//! - [`stream`] — Stream frame counters: sent, lost, recovered via PLC.
//! - [`receiver`] — Per-receiver packet counters: received, dropped, reconnects.
//! - [`network`] — Network quality counters: late packets, reorders, jitter events.
//!
//! # Design contract
//!
//! - **No heap allocation in update paths.** All update methods take `&self`
//!   and use `Relaxed` atomics — callers that need ordering guarantees
//!   (e.g. happens-before) must apply their own fences.
//! - **No I/O, no blocking, no mutex** — safe to call from near-RT paths.
//! - **Saturating counts** — counters saturate at [`u64::MAX`] rather than wrap.
//! - **Snapshots are truthful** — unknown values are exposed as `Option<u64>`,
//!   never as a fabricated zero.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]

use std::sync::atomic::{AtomicU64, Ordering};

pub mod audio;
pub mod device;
pub mod network;
pub mod receiver;
pub mod stream;

pub use audio::AudioMetrics;
pub use device::DeviceMetrics;
pub use network::NetworkMetrics;
pub use receiver::ReceiverMetrics;
pub use stream::StreamMetrics;

/// Aggregated snapshot of all bounded metrics, serialisable for API responses.
#[derive(Clone, Debug, serde::Serialize)]
pub struct MetricsSnapshot {
    /// Schema version, incremented on breaking changes.
    pub schema_version: u8,
    /// Audio backend metrics.
    pub audio: audio::AudioSnapshot,
    /// Device lifecycle metrics.
    pub device: device::DeviceSnapshot,
    /// Stream frame metrics.
    pub stream: stream::StreamSnapshot,
    /// Receiver packet metrics.
    pub receiver: receiver::ReceiverSnapshot,
    /// Network quality metrics.
    pub network: network::NetworkSnapshot,
}

/// Facade that owns one instance of every metric group.
///
/// Typical usage: one `Metrics` per server instance, shared via `Arc`.
#[derive(Debug, Default)]
pub struct Metrics {
    /// Audio backend counters.
    pub audio: AudioMetrics,
    /// Device lifecycle counters.
    pub device: DeviceMetrics,
    /// Stream frame counters.
    pub stream: StreamMetrics,
    /// Receiver packet counters.
    pub receiver: ReceiverMetrics,
    /// Network quality counters.
    pub network: NetworkMetrics,
}

impl Metrics {
    /// Create all metric groups initialised to zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a consistent point-in-time snapshot (each field read with `Acquire`).
    #[must_use]
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            schema_version: 1,
            audio: self.audio.snapshot(),
            device: self.device.snapshot(),
            stream: self.stream.snapshot(),
            receiver: self.receiver.snapshot(),
            network: self.network.snapshot(),
        }
    }
}

/// Helper: saturating atomic increment with `Relaxed` ordering.
///
/// Safe to call from any thread; callers requiring ordering guarantees must
/// issue their own fences.
#[inline]
pub(crate) fn saturating_inc(counter: &AtomicU64) {
    // Fetch current, compute saturating +1, compare-exchange.  The CAS loop
    // avoids the `fetch_update` closure syntax for older toolchains and keeps
    // the logic explicit.
    let mut current = counter.load(Ordering::Relaxed);
    loop {
        if current == u64::MAX {
            return;
        }
        match counter.compare_exchange_weak(
            current,
            current + 1,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return,
            Err(actual) => current = actual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturating_inc_does_not_wrap_at_max() {
        let c = AtomicU64::new(u64::MAX);
        saturating_inc(&c);
        assert_eq!(c.load(Ordering::Relaxed), u64::MAX);
    }

    #[test]
    fn saturating_inc_increments_from_zero() {
        let c = AtomicU64::new(0);
        saturating_inc(&c);
        assert_eq!(c.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn metrics_snapshot_schema_version_is_one() {
        let m = Metrics::new();
        assert_eq!(m.snapshot().schema_version, 1);
    }

    #[test]
    fn metrics_all_zero_on_construction() {
        let m = Metrics::new();
        let s = m.snapshot();
        assert_eq!(s.audio.xrun_count, 0);
        assert_eq!(s.audio.frames_processed, 0);
        assert_eq!(s.device.loss_events, 0);
        assert_eq!(s.device.recovery_events, 0);
        assert_eq!(s.stream.frames_sent, 0);
        assert_eq!(s.stream.frames_lost, 0);
        assert_eq!(s.stream.frames_plc_recovered, 0);
        assert_eq!(s.receiver.packets_received, 0);
        assert_eq!(s.receiver.packets_dropped, 0);
        assert_eq!(s.receiver.reconnect_count, 0);
        assert_eq!(s.network.late_packets, 0);
        assert_eq!(s.network.reordered_packets, 0);
        assert_eq!(s.network.jitter_events, 0);
    }
}
