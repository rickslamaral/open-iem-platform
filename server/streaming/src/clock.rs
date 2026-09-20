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
