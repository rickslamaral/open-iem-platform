//! Dynamics compressor configuration — Phase 2 stub.
//!
//! Detector, envelope, and gain reduction processing are deferred to Phase 7.
//! Configuration API exists now so scene/state schemas stay stable.

/// Compressor configuration. **STUB:** `process` returns input unchanged.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Compressor {
    /// Threshold in dBFS.
    pub threshold_db: f32,
    /// Compression ratio (1.0 = no compression).
    pub ratio: f32,
    /// Attack in milliseconds.
    pub attack_ms: f32,
    /// Release in milliseconds.
    pub release_ms: f32,
    /// Whether compressor is enabled.
    pub enabled: bool,
    /// Monotonic revision counter.
    pub revision: u64,
}

impl Default for Compressor {
    fn default() -> Self {
        Self::new()
    }
}
impl Compressor {
    /// Create a disabled compressor with safe defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            threshold_db: -18.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            enabled: false,
            revision: 0,
        }
    }
    /// Set enabled state.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.revision = self.revision.saturating_add(1);
    }
    /// Phase 2 passthrough. Dynamics processing arrives in Phase 7.
    #[must_use]
    #[inline]
    pub fn process(&self, left: f32, right: f32) -> (f32, f32) {
        (left, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_passthrough() {
        let c = Compressor::new();
        assert!(!c.enabled);
        assert_eq!(c.process(0.7, -0.3), (0.7, -0.3));
    }
    #[test]
    fn enabled_revision_changes() {
        let mut c = Compressor::new();
        c.set_enabled(true);
        assert_eq!(c.revision, 1);
    }
}
