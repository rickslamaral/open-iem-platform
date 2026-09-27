//! Capture-timeline timestamps, bounded drift estimation and adaptive resampling.
//!
//! All limits are explicit. This module is simulation-safe and does not perform
//! I/O; an audio backend owns clock reads and supplies sample counters.

/// Nominal sample rate used by MVP.
pub const NOMINAL_SAMPLE_RATE: u32 = 48_000;
const MAX_CORRECTION_PPM: f64 = 500.0;
const MIN_RESAMPLE_RATIO: f64 = 0.9995;
const MAX_RESAMPLE_RATIO: f64 = 1.0005;

/// Monotonic media position on capture/audio-interface timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SampleTimestamp {
    pub sequence: u64,
    pub sample: u64,
}

impl SampleTimestamp {
    #[must_use]
    pub const fn new(sequence: u64, sample: u64) -> Self {
        Self { sequence, sample }
    }
}

/// Bounded low-pass estimate of receiver clock error, in parts per million.
#[derive(Debug, Clone, Copy)]
pub struct DriftEstimator {
    nominal_rate: f64,
    last_remote: Option<u64>,
    last_local: Option<u64>,
    estimate_ppm: f64,
    smoothing: f64,
}

impl DriftEstimator {
    /// Create estimator. `smoothing` is clamped to [0, 1].
    #[must_use]
    pub fn new(nominal_rate: u32, smoothing: f64) -> Self {
        Self {
            nominal_rate: f64::from(nominal_rate.max(1)),
            last_remote: None,
            last_local: None,
            estimate_ppm: 0.0,
            smoothing: if smoothing.is_finite() {
                smoothing.clamp(0.0, 1.0)
            } else {
                0.25
            },
        }
    }

    /// Add remote capture and local output counters. First sample establishes baseline.
    #[must_use]
    pub fn update(&mut self, remote: u64, local: u64) -> f64 {
        let Some(previous_remote) = self.last_remote else {
            self.last_remote = Some(remote);
            self.last_local = Some(local);
            return 0.0;
        };
        let previous_local = self.last_local.unwrap_or(local);
        if remote <= previous_remote || local < previous_local {
            return self.estimate_ppm;
        }
        let remote_delta = remote - previous_remote;
        let local_delta = local - previous_local;
        self.last_remote = Some(remote);
        self.last_local = Some(local);
        let raw = ((local_delta as f64 / remote_delta as f64) - 1.0) * 1_000_000.0;
        let bounded = raw.clamp(-MAX_CORRECTION_PPM, MAX_CORRECTION_PPM);
        self.estimate_ppm += (bounded - self.estimate_ppm) * self.smoothing;
        self.estimate_ppm = self
            .estimate_ppm
            .clamp(-MAX_CORRECTION_PPM, MAX_CORRECTION_PPM);
        self.estimate_ppm
    }

    #[must_use]
    pub fn estimate_ppm(&self) -> f64 {
        self.estimate_ppm
    }
    #[must_use]
    pub fn nominal_rate(&self) -> f64 {
        self.nominal_rate
    }
}

/// Buffer-depth controller producing bounded adaptive resampling ratio.
#[derive(Debug, Clone, Copy)]
pub struct AdaptiveResampler {
    target_frames: usize,
    ratio: f64,
}

impl AdaptiveResampler {
    #[must_use]
    pub fn new(target_frames: usize) -> Self {
        Self {
            target_frames: target_frames.max(1),
            ratio: 1.0,
        }
    }

    /// Update ratio from clock estimate and bounded buffer error.
    #[must_use]
    pub fn update(&mut self, drift_ppm: f64, buffered_frames: usize) -> f64 {
        if !drift_ppm.is_finite() {
            return self.ratio;
        }
        let error =
            (self.target_frames as f64 - buffered_frames as f64) / self.target_frames as f64;
        let correction = (drift_ppm / 1_000_000.0 + error * 0.0005).clamp(-0.0005, 0.0005);
        self.ratio = (1.0 + correction).clamp(MIN_RESAMPLE_RATIO, MAX_RESAMPLE_RATIO);
        self.ratio
    }

    #[must_use]
    pub fn ratio(&self) -> f64 {
        self.ratio
    }
    #[must_use]
    pub fn target_frames(&self) -> usize {
        self.target_frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_uses_capture_sample_counter() {
        assert_eq!(SampleTimestamp::new(7, 48_000).sample, 48_000);
    }

    #[test]
    fn estimator_converges_and_stays_bounded() {
        let mut e = DriftEstimator::new(NOMINAL_SAMPLE_RATE, 0.25);
        let _ = e.update(0, 0);
        for n in 1..=20 {
            let _ = e.update(n * 48_000, n * 48_024);
        }
        assert!(e.estimate_ppm() > 0.0);
        assert!(e.estimate_ppm() <= MAX_CORRECTION_PPM);
        let _ = e.update(1_000_000, 10_000_000);
        assert!(e.estimate_ppm() <= MAX_CORRECTION_PPM);
    }

    #[test]
    fn estimator_ignores_timestamp_regression_and_non_finite_smoothing() {
        let mut e = DriftEstimator::new(NOMINAL_SAMPLE_RATE, f64::NAN);
        let _ = e.update(1_000, 1_000);
        let _ = e.update(900, 900);
        assert!(e.estimate_ppm().is_finite());
        assert!(e.estimate_ppm().abs() < f64::EPSILON);
    }

    #[test]
    fn resampler_ratio_stays_bounded() {
        let mut r = AdaptiveResampler::new(256);
        assert!((r.update(500.0, 0) - MAX_RESAMPLE_RATIO).abs() < f64::EPSILON);
        assert!((r.update(-500.0, 10_000) - MIN_RESAMPLE_RATIO).abs() < f64::EPSILON);
        let stable = r.ratio();
        assert!((r.update(f64::NAN, 256) - stable).abs() < f64::EPSILON);
    }

    #[test]
    fn long_run_buffer_control_does_not_run_away() {
        let mut r = AdaptiveResampler::new(256);
        for depth in (0..10_000).map(|n| 256 + usize::try_from(n % 17).expect("non-negative") - 8) {
            let ratio = r.update(120.0, depth);
            assert!((MIN_RESAMPLE_RATIO..=MAX_RESAMPLE_RATIO).contains(&ratio));
        }
    }
}

// --- additional boundary regressions ---
#[cfg(test)]
mod clock_boundary_tests {
    use super::*;

    #[test]
    fn drift_estimator_clamps_zero_nominal_rate_to_one() {
        let e = DriftEstimator::new(0, 0.5);
        let nr = e.nominal_rate();
        assert!(
            (nr - 1.0).abs() < f64::EPSILON,
            "nominal_rate 0 must be clamped to 1.0; got {nr}"
        );
    }

    #[test]
    fn drift_estimator_clamps_negative_smoothing_to_zero() {
        // smoothing clamped to 0.0: low-pass never moves, so estimate stays 0.0
        let mut e = DriftEstimator::new(NOMINAL_SAMPLE_RATE, -1.0);
        let _ = e.update(0, 0);
        let _ = e.update(48_000, 48_500);
        let ppm = e.estimate_ppm();
        assert!(
            ppm.abs() < f64::EPSILON,
            "smoothing 0.0 must freeze estimate at 0.0; got {ppm}"
        );
    }

    #[test]
    fn drift_estimator_clamps_over_one_smoothing_to_one() {
        // smoothing clamped to 1.0: estimate equals last bounded raw value exactly
        let mut e = DriftEstimator::new(NOMINAL_SAMPLE_RATE, 5.0);
        let _ = e.update(0, 0);
        let result = e.update(48_000, 48_500);
        // raw = (48500/48000 - 1) * 1e6 ≈ 10416.7 -> clamped to 500.0
        assert!(
            (result - MAX_CORRECTION_PPM).abs() < 1.0,
            "smoothing 1.0 must produce bounded raw value; got {result}"
        );
    }

    #[test]
    fn drift_estimator_first_update_always_returns_zero() {
        let mut e = DriftEstimator::new(NOMINAL_SAMPLE_RATE, 0.5);
        // First call only sets baseline, never produces a drift measurement.
        let first = e.update(0, 0);
        let second = e.update(48_000, 48_048);
        assert!(
            first.abs() < f64::EPSILON,
            "first update must return 0.0; got {first}"
        );
        assert!(
            (second - e.estimate_ppm()).abs() < f64::EPSILON,
            "second update must return current estimate; got {second}"
        );
    }

    #[test]
    fn drift_estimator_stale_remote_returns_current_estimate() {
        let mut e = DriftEstimator::new(NOMINAL_SAMPLE_RATE, 0.5);
        let _ = e.update(0, 0);
        let after_first = e.update(48_000, 48_024);
        // Replay same remote counter -> stale branch, estimate unchanged.
        let after_stale = e.update(48_000, 48_100);
        assert!(
            (after_stale - after_first).abs() < f64::EPSILON,
            "stale remote must not change estimate; first={after_first} stale={after_stale}"
        );
    }

    #[test]
    fn drift_estimator_local_regression_returns_current_estimate() {
        let mut e = DriftEstimator::new(NOMINAL_SAMPLE_RATE, 0.5);
        let _ = e.update(0, 0);
        let after_first = e.update(48_000, 48_024);
        // local goes backwards -> ignored, estimate unchanged.
        let after_regression = e.update(96_000, 47_000);
        assert!(
            (after_regression - after_first).abs() < f64::EPSILON,
            "local regression must not change estimate; first={after_first} regression={after_regression}"
        );
    }

    #[test]
    fn adaptive_resampler_zero_target_clamped_to_one() {
        let r = AdaptiveResampler::new(0);
        assert_eq!(r.target_frames(), 1, "target_frames 0 must be clamped to 1");
    }

    #[test]
    fn adaptive_resampler_initial_ratio_is_one() {
        let r = AdaptiveResampler::new(256);
        let ratio = r.ratio();
        assert!(
            (ratio - 1.0).abs() < f64::EPSILON,
            "initial ratio must be 1.0 before any update; got {ratio}"
        );
    }

    #[test]
    fn sample_timestamp_sequence_field_preserved() {
        let ts = SampleTimestamp::new(42, 0);
        assert_eq!(ts.sequence, 42);
    }
}

// --- stagnant-local and extreme-buffered regressions ---
#[cfg(test)]
mod clock_extra_boundary_tests {
    use super::*;

    /// When local counter stays frozen while remote advances, `local_delta` is
    /// zero. `raw = (0/remote_delta - 1) * 1e6 = -1_000_000`, clamped to
    /// `-MAX_CORRECTION_PPM`. The low-pass filter must move the estimate toward
    /// that bound and the final value must remain finite and bounded.
    #[test]
    fn drift_estimator_stagnant_local_clamps_to_negative_max() {
        let mut e = DriftEstimator::new(NOMINAL_SAMPLE_RATE, 1.0); // smoothing=1: instant adoption
                                                                   // Establish baseline.
        let _ = e.update(0, 0);
        // Remote advances by 1000 samples; local stays at 0 — output stall scenario.
        let result = e.update(1_000, 0);
        // smoothing=1 → estimate = bounded raw = -MAX_CORRECTION_PPM
        assert!(
            (result - (-MAX_CORRECTION_PPM)).abs() < 1.0,
            "stagnant local must produce estimate near -MAX_CORRECTION_PPM; got {result}"
        );
        assert!(result.is_finite(), "estimate must remain finite");
        assert!(result >= -MAX_CORRECTION_PPM, "must not exceed lower bound");
    }

    /// `AdaptiveResampler::update` with `buffered_frames` far above `target`
    /// produces a very negative error; the correction is clamped before being
    /// added to 1.0, so the ratio must reach `MIN_RESAMPLE_RATIO` but not fall
    /// below it.
    #[test]
    fn adaptive_resampler_extreme_overshoot_stays_at_minimum_ratio() {
        let mut r = AdaptiveResampler::new(256);
        // Drive with 10_000 buffered frames (40x overshoot) at zero drift ppm.
        let ratio = r.update(0.0, 10_000);
        assert!(
            (ratio - MIN_RESAMPLE_RATIO).abs() < f64::EPSILON,
            "extreme overshoot must clamp to MIN_RESAMPLE_RATIO; got {ratio}"
        );
    }

    /// `AdaptiveResampler::update` with zero buffered frames (buffer drained
    /// completely) must reach `MAX_RESAMPLE_RATIO` regardless of drift ppm.
    #[test]
    fn adaptive_resampler_extreme_undershoot_stays_at_maximum_ratio() {
        let mut r = AdaptiveResampler::new(256);
        let ratio = r.update(0.0, 0);
        assert!(
            (ratio - MAX_RESAMPLE_RATIO).abs() < f64::EPSILON,
            "empty buffer must clamp to MAX_RESAMPLE_RATIO; got {ratio}"
        );
    }
}
