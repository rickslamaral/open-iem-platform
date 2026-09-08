//! Limiter — output brick-wall limiter stub.
//!
//! This is a stub implementation that defines the limiter configuration and
//! provides a placeholder `process` method. The full lookahead limiter
//! algorithm will be implemented in Phase 2.
//!
//! # Safety
//!
//! The `process` method is realtime-safe: no allocation, no I/O, no blocking.

use crate::GAIN_DB_MIN;

/// Brick-wall limiter configuration.
///
/// # Phase 1 Status: STUB
///
/// The `process` method currently passes audio through unchanged when disabled,
/// or applies a simple hard-clip at the threshold when enabled.
/// A proper lookahead limiter will replace this in Phase 2.
#[derive(Debug, Clone, PartialEq)]
pub struct Limiter {
    /// Threshold in dB. No sample should exceed this level at output.
    ///
    /// Typical value: -0.3 dBFS (leaves headroom for inter-sample peaks).
    /// Bounded to `[GAIN_DB_MIN, 0.0]`.
    threshold_db: f32,

    /// Whether the limiter is active.
    pub enabled: bool,

    /// Monotonic revision counter.
    revision: u64,
}

impl Limiter {
    /// Default threshold: -0.3 dBFS.
    pub const DEFAULT_THRESHOLD_DB: f32 = -0.3;

    /// Create a new limiter with default threshold and disabled.
    ///
    /// # Safety Note
    ///
    /// The limiter is disabled by default. Enable explicitly after confirming
    /// the mix chain is producing audio — do not rely on the limiter as the
    /// primary gain management mechanism.
    #[must_use]
    pub fn new() -> Self {
        Self {
            threshold_db: Self::DEFAULT_THRESHOLD_DB,
            enabled: false,
            revision: 0,
        }
    }

    /// Create a new limiter enabled with the given threshold.
    #[must_use]
    pub fn new_enabled(threshold_db: f32) -> Self {
        let mut l = Self::new();
        l.set_threshold_db(threshold_db);
        l.set_enabled(true);
        l
    }

    /// Returns the threshold in dB.
    #[must_use]
    pub fn threshold_db(&self) -> f32 {
        self.threshold_db
    }

    /// Returns the threshold as a linear amplitude value.
    #[must_use]
    #[inline]
    pub fn threshold_linear(&self) -> f32 {
        crate::db_to_linear(self.threshold_db)
    }

    /// Returns the current monotonic revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Set the threshold in dB.
    ///
    /// Clamped to `[GAIN_DB_MIN, 0.0]`. Increments revision.
    pub fn set_threshold_db(&mut self, db: f32) {
        self.threshold_db = db.clamp(GAIN_DB_MIN, 0.0);
        self.revision = self.revision.saturating_add(1);
    }

    /// Set the enabled state. Increments revision.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.revision = self.revision.saturating_add(1);
    }

    /// Process a stereo sample pair through the limiter.
    ///
    /// # Phase 1 Stub Behaviour
    ///
    /// - If disabled: pass-through (no processing).
    /// - If enabled: hard-clip both channels to `[-threshold_linear, +threshold_linear]`.
    ///
    /// **This is NOT a production limiter.** It has no lookahead, no gain
    /// reduction smoothing, and will introduce harmonic distortion on transients.
    /// Replace with a proper peak limiter in Phase 2.
    ///
    /// # Realtime Safety
    ///
    /// No allocation, no I/O, no blocking. Safe to call from audio callback.
    ///
    /// # Arguments
    ///
    /// * `left` - Left channel sample (-1.0 to +1.0 nominal range).
    /// * `right` - Right channel sample.
    ///
    /// # Returns
    ///
    /// `(left_out, right_out)` — clamped samples.
    #[must_use]
    #[inline]
    pub fn process(&self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        let thresh = self.threshold_linear();
        (left.clamp(-thresh, thresh), right.clamp(-thresh, thresh))
    }
}

impl Default for Limiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter_defaults() {
        let lim = Limiter::new();
        assert!(!lim.enabled);
        assert!((lim.threshold_db() - Limiter::DEFAULT_THRESHOLD_DB).abs() < f32::EPSILON);
        assert_eq!(lim.revision(), 0);
    }

    #[test]
    fn test_limiter_passthrough_when_disabled() {
        let lim = Limiter::new();
        let (l, r) = lim.process(2.0, -3.0);
        assert!((l - 2.0).abs() < f32::EPSILON);
        assert!((r - (-3.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_limiter_clips_when_enabled() {
        let lim = Limiter::new_enabled(-0.3);
        let thresh = lim.threshold_linear();
        let (l, r) = lim.process(2.0, -2.0);
        assert!(
            (l - thresh).abs() < 1e-5,
            "left should be clamped to threshold"
        );
        assert!(
            (r - (-thresh)).abs() < 1e-5,
            "right should be clamped to -threshold"
        );
    }

    #[test]
    fn test_limiter_passes_normal_signal() {
        let lim = Limiter::new_enabled(-0.3);
        let (l, r) = lim.process(0.5, -0.5);
        // 0.5 is well below threshold (~0.983)
        assert!((l - 0.5).abs() < 1e-5);
        assert!((r - (-0.5)).abs() < 1e-5);
    }

    #[test]
    fn test_limiter_threshold_clamped() {
        let mut lim = Limiter::new();
        lim.set_threshold_db(10.0); // above 0 dB — should clamp to 0
        assert!((lim.threshold_db() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_limiter_revision_increments() {
        let mut lim = Limiter::new();
        assert_eq!(lim.revision(), 0);
        lim.set_enabled(true);
        assert_eq!(lim.revision(), 1);
        lim.set_threshold_db(-6.0);
        assert_eq!(lim.revision(), 2);
    }

    #[test]
    fn test_limiter_threshold_linear() {
        let lim = Limiter::new_enabled(0.0);
        // 0 dBFS = linear 1.0
        assert!((lim.threshold_linear() - 1.0).abs() < 1e-5);
    }
}
