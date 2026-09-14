//! Parametric EQ — Phase 7: real biquad DSP processing.
//!
//! Type-II transposed Direct Form II (TDF2) peaking biquad sections.
//! Coefficients follow the Audio EQ Cookbook (Robert Bristow-Johnson).

use std::f32::consts::PI;

/// Sample rate assumed for coefficient computation.
pub const SAMPLE_RATE: f32 = 48_000.0;

/// Maximum supported parametric EQ bands.
pub const MAX_EQ_BANDS: usize = 4;

// ---------------------------------------------------------------------------
// BiquadCoeffs
// ---------------------------------------------------------------------------

/// Biquad filter coefficients for a peaking EQ section.
///
/// Transfer function: H(z) = (b0 + b1·z⁻¹ + b2·z⁻²) / (1 + a1·z⁻¹ + a2·z⁻²)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiquadCoeffs {
    /// Feed-forward coefficient b0 (normalised by a0).
    pub b0: f32,
    /// Feed-forward coefficient b1 (normalised by a0).
    pub b1: f32,
    /// Feed-forward coefficient b2 (normalised by a0).
    pub b2: f32,
    /// Feedback coefficient a1 (normalised by a0).
    pub a1: f32,
    /// Feedback coefficient a2 (normalised by a0).
    pub a2: f32,
}

impl BiquadCoeffs {
    /// Identity (passthrough) coefficients.
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    /// Compute peaking EQ coefficients (RBJ Audio EQ Cookbook).
    #[must_use]
    pub fn peaking(frequency_hz: f32, gain_db: f32, q: f32) -> Self {
        // A = sqrt(10^(dB/20)) = 10^(dB/40)
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * frequency_hz / SAMPLE_RATE;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

// ---------------------------------------------------------------------------
// BiquadState
// ---------------------------------------------------------------------------

/// Per-band stereo state for type-II transposed Direct Form II.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiquadState {
    /// Left-channel delay-line element 1.
    pub w1_l: f32,
    /// Left-channel delay-line element 2.
    pub w2_l: f32,
    /// Right-channel delay-line element 1.
    pub w1_r: f32,
    /// Right-channel delay-line element 2.
    pub w2_r: f32,
}

impl BiquadState {
    /// Return zeroed filter state.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            w1_l: 0.0,
            w2_l: 0.0,
            w1_r: 0.0,
            w2_r: 0.0,
        }
    }

    /// Process one stereo sample pair through the biquad (transposed DF2).
    #[inline]
    pub fn tick(&mut self, c: &BiquadCoeffs, left: f32, right: f32) -> (f32, f32) {
        let out_l = c.b0 * left + self.w1_l;
        self.w1_l = c.b1 * left - c.a1 * out_l + self.w2_l;
        self.w2_l = c.b2 * left - c.a2 * out_l;

        let out_r = c.b0 * right + self.w1_r;
        self.w1_r = c.b1 * right - c.a1 * out_r + self.w2_r;
        self.w2_r = c.b2 * right - c.a2 * out_r;

        (out_l, out_r)
    }
}

// ---------------------------------------------------------------------------
// EqBand
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// ParametricEq
// ---------------------------------------------------------------------------

/// Fixed-capacity parametric EQ with real biquad DSP. No heap allocation.
///
/// `bands` mirrors current configuration; DSP state lives in private arrays.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParametricEq {
    /// Public band configurations (API-stable field).
    pub bands: [EqBand; MAX_EQ_BANDS],
    /// Monotonic revision counter — incremented on every `set_band` call.
    pub revision: u64,
    /// Precomputed biquad coefficients, one per band.
    coeffs: [BiquadCoeffs; MAX_EQ_BANDS],
    /// Per-band stereo filter state (inline, no heap).
    state: [BiquadState; MAX_EQ_BANDS],
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
        const DEFAULT_BAND: EqBand = EqBand {
            frequency_hz: 1_000.0,
            gain_db: 0.0,
            q: 1.0,
            enabled: false,
        };
        Self {
            bands: [DEFAULT_BAND; MAX_EQ_BANDS],
            revision: 0,
            coeffs: [BiquadCoeffs::identity(); MAX_EQ_BANDS],
            state: [BiquadState::zero(); MAX_EQ_BANDS],
        }
    }

    /// Replace band configuration. Index outside capacity is silently ignored.
    ///
    /// Recomputes biquad coefficients immediately and resets filter state for
    /// the changed band to prevent transients from stale delay-line values.
    pub fn set_band(&mut self, index: usize, band: EqBand) {
        if index < MAX_EQ_BANDS {
            self.bands[index] = band;
            self.coeffs[index] = if band.enabled {
                BiquadCoeffs::peaking(band.frequency_hz, band.gain_db, band.q)
            } else {
                BiquadCoeffs::identity()
            };
            self.state[index] = BiquadState::zero();
            self.revision = self.revision.saturating_add(1);
        }
    }

    /// Process one stereo sample through all enabled biquad bands in series.
    #[inline]
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        let mut l = left;
        let mut r = right;
        for i in 0..MAX_EQ_BANDS {
            if self.bands[i].enabled {
                let (nl, nr) = self.state[i].tick(&self.coeffs[i], l, r);
                l = nl;
                r = nr;
            }
        }
        (l, r)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn rms_left(samples: &[(f32, f32)]) -> f32 {
        let sum: f32 = samples.iter().map(|(l, _)| l * l).sum();
        (sum / samples.len() as f32).sqrt()
    }

    fn sinusoid_pairs(freq_hz: f32, n: usize) -> Vec<(f32, f32)> {
        (0..n)
            .map(|i| {
                let s = (2.0 * PI * freq_hz * i as f32 / SAMPLE_RATE).sin();
                (s, s)
            })
            .collect()
    }

    // --- existing tests (preserved) -----------------------------------------

    #[test]
    fn defaults_flat_and_passthrough() {
        let mut eq = ParametricEq::new();
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

    // --- new Phase 7 tests --------------------------------------------------

    #[test]
    fn gain_0db_passthrough() {
        let mut eq = ParametricEq::new();
        eq.set_band(
            0,
            EqBand {
                frequency_hz: 1_000.0,
                gain_db: 0.0,
                q: 1.0,
                enabled: true,
            },
        );
        let (l, r) = eq.process(0.5, -0.3);
        assert!(
            (l - 0.5).abs() < 1e-5,
            "left deviation: {}",
            (l - 0.5).abs()
        );
        assert!(
            (r - (-0.3)).abs() < 1e-5,
            "right deviation: {}",
            (r - (-0.3)).abs()
        );
    }

    #[test]
    fn disabled_band_passthrough() {
        let mut eq = ParametricEq::new();
        eq.set_band(
            0,
            EqBand {
                frequency_hz: 1_000.0,
                gain_db: 6.0,
                q: 1.0,
                enabled: false,
            },
        );
        let (l, r) = eq.process(0.5, -0.3);
        assert!((l - 0.5).abs() < f32::EPSILON);
        assert!((r - (-0.3)).abs() < f32::EPSILON);
    }

    #[test]
    fn boost_increases_output_at_freq() {
        // Baseline RMS of raw 1 kHz sinusoid.
        let bypass_rms = rms_left(&sinusoid_pairs(1_000.0, 4800));

        let mut eq = ParametricEq::new();
        eq.set_band(
            0,
            EqBand {
                frequency_hz: 1_000.0,
                gain_db: 6.0,
                q: 1.0,
                enabled: true,
            },
        );
        let boosted: Vec<(f32, f32)> = sinusoid_pairs(1_000.0, 4800)
            .into_iter()
            .map(|(l, r)| eq.process(l, r))
            .collect();
        let boosted_rms = rms_left(&boosted);

        assert!(
            boosted_rms > bypass_rms * 1.5,
            "expected boost: bypass={bypass_rms:.4} boosted={boosted_rms:.4}"
        );
    }

    #[test]
    fn cut_decreases_output_at_freq() {
        let bypass_rms = rms_left(&sinusoid_pairs(1_000.0, 4800));

        let mut eq = ParametricEq::new();
        eq.set_band(
            0,
            EqBand {
                frequency_hz: 1_000.0,
                gain_db: -6.0,
                q: 1.0,
                enabled: true,
            },
        );
        let cut: Vec<(f32, f32)> = sinusoid_pairs(1_000.0, 4800)
            .into_iter()
            .map(|(l, r)| eq.process(l, r))
            .collect();
        let cut_rms = rms_left(&cut);

        assert!(
            cut_rms < bypass_rms * 0.8,
            "expected cut: bypass={bypass_rms:.4} cut={cut_rms:.4}"
        );
    }

    #[test]
    fn two_bands_independent() {
        let mut eq = ParametricEq::new();
        eq.set_band(
            0,
            EqBand {
                frequency_hz: 200.0,
                gain_db: 3.0,
                q: 1.0,
                enabled: true,
            },
        );
        eq.set_band(
            1,
            EqBand {
                frequency_hz: 4_000.0,
                gain_db: -3.0,
                q: 1.0,
                enabled: true,
            },
        );
        assert_eq!(eq.revision, 2);
    }

    #[test]
    fn set_band_out_of_range_ignored() {
        let mut eq = ParametricEq::new();
        eq.set_band(4, EqBand::default()); // index 4 is out of [0..4)
        assert_eq!(eq.revision, 0);
    }

    #[test]
    fn process_takes_mut_self() {
        // Compile-time proof: process requires &mut self.
        let mut eq = ParametricEq::new();
        let _ = eq.process(0.1, 0.2);
    }

    #[test]
    fn coefficients_match_reference_vectors() {
        // Reference values generated independently with the RBJ Audio EQ
        // Cookbook equations at 48 kHz. Tolerance covers f32 rounding.
        let cases: &[(f32, f32, f32, [f64; 5])] = &[
            (
                100.0,
                6.0,
                0.707,
                [
                    1.006480037107,
                    -1.986808003509,
                    0.980498195648,
                    -1.986808003509,
                    0.986978232755,
                ],
            ),
            (
                1_000.0,
                -3.0,
                1.0,
                [
                    0.978977346094,
                    -1.840157304773,
                    0.877058603523,
                    -1.840157304773,
                    0.856035949617,
                ],
            ),
            (
                12_000.0,
                9.0,
                2.5,
                [1.193568133102, 0.0, 0.593530469976, 0.0, 0.787098603079],
            ),
            (
                20_000.0,
                -12.0,
                0.5,
                [
                    0.626038301479,
                    0.867052359030,
                    0.375147524296,
                    0.867052359030,
                    0.001185825775,
                ],
            ),
        ];
        for &(frequency, gain, q, expected) in cases {
            let actual = BiquadCoeffs::peaking(frequency, gain, q);
            for (name, got, want) in [
                ("b0", f64::from(actual.b0), expected[0]),
                ("b1", f64::from(actual.b1), expected[1]),
                ("b2", f64::from(actual.b2), expected[2]),
                ("a1", f64::from(actual.a1), expected[3]),
                ("a2", f64::from(actual.a2), expected[4]),
            ] {
                assert!(
                    (got - want).abs() < 5e-6,
                    "{name} mismatch for {frequency} Hz/{gain} dB/Q {q}: got {got}, want {want}"
                );
            }
        }
    }

    #[test]
    fn coefficients_stability() {
        let cases: &[(f32, f32, f32)] = &[
            (1_000.0, 12.0, 0.1),
            (1_000.0, 12.0, 10.0),
            (1_000.0, -12.0, 0.1),
            (1_000.0, -12.0, 10.0),
        ];
        for &(freq, gain, q) in cases {
            let mut eq = ParametricEq::new();
            eq.set_band(
                0,
                EqBand {
                    frequency_hz: freq,
                    gain_db: gain,
                    q,
                    enabled: true,
                },
            );
            let (l, r) = eq.process(0.5, 0.5);
            assert!(
                !l.is_nan() && !l.is_infinite(),
                "NaN/Inf left: freq={freq} gain={gain} q={q} → l={l}"
            );
            assert!(
                !r.is_nan() && !r.is_infinite(),
                "NaN/Inf right: freq={freq} gain={gain} q={q} → r={r}"
            );
        }
    }
}
