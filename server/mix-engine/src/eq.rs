//! Parametric EQ configuration — Phase 2 stub.
//!
//! Coefficient design and biquad processing are deferred to Phase 7. This
//! module fixes realtime-safe control-plane API now; `process` is passthrough.

/// Maximum supported parametric EQ bands.
pub const MAX_EQ_BANDS: usize = 4;

/// One parametric EQ band configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EqBand {
    /// Centre frequency in Hz.
    pub frequency_hz: f32,
    /// Gain in dB.
    pub gain_db: f32,
    /// Q factor.
    pub q: f32,
    /// Whether band is active.
    pub enabled: bool,
}

impl Default for EqBand {
    fn default() -> Self {
        Self {
            frequency_hz: 1_000.0,
            gain_db: 0.0,
            q: 1.0,
            enabled: false,
        }
    }
}

/// Fixed-capacity parametric EQ. **STUB:** processing is passthrough until Phase 7.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParametricEq {
    /// Configured bands.
    pub bands: [EqBand; MAX_EQ_BANDS],
    /// Monotonic revision counter.
    pub revision: u64,
}

impl Default for ParametricEq {
    fn default() -> Self {
        Self::new()
    }
}
impl ParametricEq {
    /// Create disabled, flat EQ.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bands: [EqBand {
                frequency_hz: 1_000.0,
                gain_db: 0.0,
                q: 1.0,
                enabled: false,
            }; MAX_EQ_BANDS],
            revision: 0,
        }
    }
    /// Replace band configuration. Index outside capacity is ignored.
    pub fn set_band(&mut self, index: usize, band: EqBand) {
        if index < MAX_EQ_BANDS {
            self.bands[index] = band;
            self.revision = self.revision.saturating_add(1);
        }
    }
    /// Phase 2 passthrough. Biquad processing arrives in Phase 7.
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
    fn defaults_flat_and_passthrough() {
        let eq = ParametricEq::new();
        assert!(!eq.bands[0].enabled);
        assert_eq!(eq.process(0.4, -0.2), (0.4, -0.2));
    }
    #[test]
    fn band_revision_changes() {
        let mut eq = ParametricEq::new();
        eq.set_band(
            0,
            EqBand {
                enabled: true,
                ..EqBand::default()
            },
        );
        assert_eq!(eq.revision, 1);
    }
}
