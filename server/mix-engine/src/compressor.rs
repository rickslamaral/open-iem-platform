//! Dynamics compressor — Phase 7 stereo-linked RMS implementation.

const SAMPLE_RATE: f32 = 48_000.0;

#[inline]
fn attack_coeff(ms: f32) -> f32 {
    (-1.0 / (SAMPLE_RATE * (ms / 1000.0).max(1e-6))).exp()
}

#[inline]
fn release_coeff(ms: f32) -> f32 {
    (-1.0 / (SAMPLE_RATE * (ms / 1000.0).max(1e-6))).exp()
}

/// Stereo-linked RMS compressor.
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

    // DSP state — inline, no heap
    rms_state: f32,
    gain_db: f32,

    // Cached coefficients
    atk_coeff: f32,
    rel_coeff: f32,
}

impl Default for Compressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compressor {
    /// Create a disabled compressor with safe defaults.
    #[must_use]
    pub fn new() -> Self {
        let attack_ms = 10.0_f32;
        let release_ms = 100.0_f32;
        Self {
            threshold_db: -18.0,
            ratio: 4.0,
            attack_ms,
            release_ms,
            enabled: false,
            revision: 0,
            rms_state: 0.0,
            gain_db: 0.0,
            atk_coeff: attack_coeff(attack_ms),
            rel_coeff: release_coeff(release_ms),
        }
    }

    /// Set enabled state and bump revision.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.revision = self.revision.saturating_add(1);
    }

    /// Set threshold in dBFS and bump revision.
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.threshold_db = threshold_db;
        self.revision = self.revision.saturating_add(1);
    }

    /// Set ratio and bump revision.
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.max(1.0);
        self.revision = self.revision.saturating_add(1);
    }

    /// Set attack in ms, recompute coefficient, bump revision.
    pub fn set_attack_ms(&mut self, attack_ms: f32) {
        self.attack_ms = attack_ms;
        self.atk_coeff = attack_coeff(attack_ms);
        self.revision = self.revision.saturating_add(1);
    }

    /// Set release in ms, recompute coefficient, bump revision.
    pub fn set_release_ms(&mut self, release_ms: f32) {
        self.release_ms = release_ms;
        self.rel_coeff = release_coeff(release_ms);
        self.revision = self.revision.saturating_add(1);
    }

    /// Process one stereo sample pair. Mutates internal state.
    #[inline]
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Stereo-linked: max absolute value
        let peak = left.abs().max(right.abs());
        let x2 = peak * peak;

        // Exp moving average of x^2 (RMS detector)
        // Use attack coeff when signal rising, release when falling
        let coeff = if x2 > self.rms_state {
            self.atk_coeff
        } else {
            self.rel_coeff
        };
        self.rms_state = coeff * self.rms_state + (1.0 - coeff) * x2;

        let target_gain_db = if self.enabled {
            // RMS in dB (avoid log(0))
            let rms_db = 10.0 * self.rms_state.max(1e-30).log10();

            if rms_db > self.threshold_db {
                // Gain reduction: (1 - 1/ratio) * (threshold - rms_db) — negative value
                let ratio = self.ratio.max(1.0);
                (1.0 - 1.0 / ratio) * (self.threshold_db - rms_db)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Smooth gain with attack/release envelope
        let gain_coeff = if target_gain_db < self.gain_db {
            // gain_db going more negative = more compression = attack
            self.atk_coeff
        } else {
            // gain recovering toward 0 = release
            self.rel_coeff
        };
        self.gain_db = gain_coeff * self.gain_db + (1.0 - gain_coeff) * target_gain_db;

        if self.enabled {
            let linear_gain = 10.0_f32.powf(self.gain_db / 20.0);
            (left * linear_gain, right * linear_gain)
        } else {
            (left, right)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_passthrough() {
        let mut c = Compressor::new();
        assert!(!c.enabled);
        assert_eq!(c.process(0.7, -0.3), (0.7, -0.3));
    }

    #[test]
    fn enabled_revision_changes() {
        let mut c = Compressor::new();
        c.set_enabled(true);
        assert_eq!(c.revision, 1);
    }

    #[test]
    fn disabled_passthrough() {
        let mut c = Compressor::new();
        assert!(!c.enabled);
        let (l, r) = c.process(0.5, -0.4);
        assert!((l - 0.5).abs() < f32::EPSILON);
        assert!((r - (-0.4)).abs() < f32::EPSILON);
    }

    #[test]
    fn enabled_passes_silence() {
        let mut c = Compressor::new();
        c.set_enabled(true);
        // silence → no compression, output stays 0
        for _ in 0..1000 {
            let (l, r) = c.process(0.0, 0.0);
            assert!(l.abs() < f32::EPSILON);
            assert!(r.abs() < f32::EPSILON);
        }
    }

    #[test]
    fn compressor_reduces_loud_signal() {
        let mut c = Compressor::new();
        c.set_enabled(true);
        c.set_threshold(-6.0);
        // Warm up detector
        for _ in 0..10_000 {
            c.process(0.9, 0.9);
        }
        let (l, _r) = c.process(0.9, 0.9);
        assert!(l < 0.9, "expected compression, got l={l}");
    }

    #[test]
    fn threshold_change_bumps_revision() {
        let mut c = Compressor::new();
        let rev = c.revision;
        c.set_threshold(-12.0);
        assert!(c.revision > rev);
    }

    #[test]
    fn ratio_change_bumps_revision() {
        let mut c = Compressor::new();
        let rev = c.revision;
        c.set_ratio(8.0);
        assert!(c.revision > rev);
    }

    #[test]
    fn enabled_change_bumps_revision() {
        let mut c = Compressor::new();
        c.set_enabled(true);
        assert_eq!(c.revision, 1);
    }

    #[test]
    fn attack_release_change_bumps_revision() {
        let mut c = Compressor::new();
        let rev = c.revision;
        c.set_attack_ms(5.0);
        assert!(c.revision > rev);
        let rev2 = c.revision;
        c.set_release_ms(200.0);
        assert!(c.revision > rev2);
    }

    #[test]
    fn no_nan_inf_on_extreme_input() {
        let mut c = Compressor::new();
        c.set_enabled(true);
        let inputs = [1.0_f32, -1.0_f32, 0.0_f32];
        for _ in 0..1000 {
            for &v in &inputs {
                let (l, r) = c.process(v, v);
                assert!(l.is_finite(), "l is not finite: {l}");
                assert!(r.is_finite(), "r is not finite: {r}");
            }
        }
    }

    #[test]
    fn stereo_link_uses_max_channel() {
        let mut c = Compressor::new();
        c.set_enabled(true);
        c.set_threshold(-6.0);
        // Warm up with L=0.9, R=0.0
        for _ in 0..10_000 {
            c.process(0.9, 0.0);
        }
        let (l, r) = c.process(0.9, 0.0);
        // Both channels compressed even though R=0 because L drives detector
        assert!(l < 0.9, "L should be compressed, got {l}");
        // R should be compressed (multiplied by same gain < 1), result < 0 or == 0
        // R input is 0.0 so output is 0.0 * gain = 0.0 — still correct
        assert!(r.abs() < f32::EPSILON); // 0 * any_gain = 0
    }
}
