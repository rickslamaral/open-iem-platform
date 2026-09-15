//! Audio backend metrics: XRUN counter and frames-processed counter.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::saturating_inc;

/// Atomic counters for the audio backend.
#[derive(Debug, Default)]
pub struct AudioMetrics {
    /// Number of XRUN (buffer underrun/overrun) events since last reset.
    xrun_count: AtomicU64,
    /// Number of audio frames processed since last reset.
    frames_processed: AtomicU64,
}

/// Point-in-time snapshot of audio backend counters.
#[derive(Clone, Debug, serde::Serialize)]
pub struct AudioSnapshot {
    /// Total XRUN events since last reset.
    pub xrun_count: u64,
    /// Total frames processed since last reset.
    pub frames_processed: u64,
}

impl AudioMetrics {
    /// Record one XRUN event.  Safe to call from any thread.
    pub fn record_xrun(&self) {
        saturating_inc(&self.xrun_count);
    }

    /// Record one processed audio frame.  Safe to call from any thread.
    pub fn record_frame(&self) {
        saturating_inc(&self.frames_processed);
    }

    /// Reset both counters to zero.
    ///
    /// Not atomic with respect to concurrent updates; intended for
    /// maintenance/test use only.
    pub fn reset(&self) {
        self.xrun_count.store(0, Ordering::Relaxed);
        self.frames_processed.store(0, Ordering::Relaxed);
    }

    /// Return a consistent snapshot.
    #[must_use]
    pub fn snapshot(&self) -> AudioSnapshot {
        AudioSnapshot {
            xrun_count: self.xrun_count.load(Ordering::Acquire),
            frames_processed: self.frames_processed.load(Ordering::Acquire),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xrun_increments() {
        let m = AudioMetrics::default();
        m.record_xrun();
        m.record_xrun();
        assert_eq!(m.snapshot().xrun_count, 2);
    }

    #[test]
    fn frames_processed_increments() {
        let m = AudioMetrics::default();
        for _ in 0..100 {
            m.record_frame();
        }
        assert_eq!(m.snapshot().frames_processed, 100);
    }

    #[test]
    fn reset_clears_both_counters() {
        let m = AudioMetrics::default();
        m.record_xrun();
        m.record_frame();
        m.reset();
        let s = m.snapshot();
        assert_eq!(s.xrun_count, 0);
        assert_eq!(s.frames_processed, 0);
    }

    #[test]
    fn saturates_at_max() {
        let m = AudioMetrics::default();
        m.xrun_count
            .store(u64::MAX, std::sync::atomic::Ordering::Relaxed);
        m.record_xrun();
        assert_eq!(m.snapshot().xrun_count, u64::MAX);
    }
}
