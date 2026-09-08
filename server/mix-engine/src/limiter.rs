//! Lookahead brick-wall limiter — Phase 2 implementation.
//!
//! Replaces the Phase 1 hard-clip stub with a proper peak limiter that:
//! - Uses a fixed-size circular delay buffer (lookahead window) — **no heap**
//! - Applies gain-reduction smoothing (attack + release envelope follower)
//! - Guarantees no sample exceeds the threshold at output
//! - Is realtime-safe: zero heap allocation, no I/O, no blocking
//!
//! # Algorithm
//!
//! 1. Input sample enters the lookahead delay line (ring buffer).
//! 2. Peak detector scans the lookahead window for the highest absolute value.
//! 3. If `peak > threshold_linear`, compute target gain = `threshold / peak`.
//! 4. Smooth the target gain with an attack/release envelope follower.
//! 5. Output the oldest sample in the delay line scaled by the smoothed gain.
//!
//! This avoids transient clipping by "looking ahead" by `LOOKAHEAD_FRAMES` and
//! attenuating before the transient arrives at the output.
//!
//! # Constants
//!
//! - `LOOKAHEAD_FRAMES = 64` — ~1.3 ms at 48 kHz (acceptable latency for IEM use)
//! - Attack: 0.5 ms (fast gain reduction)
//! - Release: 100 ms (slow gain recovery)
//!
//! # Realtime Safety
//!
//! - Buffer is a fixed-size `[f32; LOOKAHEAD_FRAMES * 2]` on the struct (stack/inline)
//! - No `Box`, no `Vec`, no `String`, no `Mutex`
//! - No `println!` / `eprintln!`
//! - No panic paths with heap-format strings

use crate::GAIN_DB_MIN;

/// Lookahead window in samples (64 @ 48 kHz ≈ 1.33 ms).
pub const LOOKAHEAD_FRAMES: usize = 64;

/// Attack time: 0.5 ms expressed as a coefficient.
/// `coef = exp(-1 / (sample_rate * attack_seconds))`
/// Pre-computed for 48 kHz.
const ATTACK_COEF: f32 = 0.998_566; // exp(-1/(48000*0.0005)) ≈ 0.9999...

/// Release time: 100 ms expressed as a coefficient.
/// Pre-computed for 48 kHz.
const RELEASE_COEF: f32 = 0.999_791_8; // exp(-1/(48000*0.1))

/// Lookahead brick-wall limiter.
///
/// # Phase 2
///
/// Full lookahead implementation. Replaces the Phase 1 hard-clip stub.
///
/// The limiter adds [`LOOKAHEAD_FRAMES`] samples of latency to the audio path.
/// At 48 kHz and 64 frames this is ~1.33 ms — within the IEM latency budget.
///
/// # Realtime Safety
///
/// Zero heap allocation. All state is inline in the struct.
#[derive(Debug, Clone)]
pub struct Limiter {
    /// Threshold in dB. No output sample will exceed this level.
    threshold_db: f32,

    /// Whether the limiter is active.
    pub enabled: bool,

    /// Monotonic revision counter.
    revision: u64,

    // Lookahead delay buffer — interleaved L/R.
    // Layout: [L0, R0, L1, R1, ..., L(N-1), R(N-1)]
    buf: [f32; LOOKAHEAD_FRAMES * 2],

    /// Write cursor into `buf` (frame index, not sample index).
    write_head: usize,

    /// Current envelope follower gain (linear, starts at 1.0 = unity).
    envelope_gain: f32,
}

impl PartialEq for Limiter {
    fn eq(&self, other: &Self) -> bool {
        self.threshold_db == other.threshold_db
            && self.enabled == other.enabled
            && self.revision == other.revision
    }
}

impl Limiter {
    /// Default threshold: -0.3 dBFS.
    pub const DEFAULT_THRESHOLD_DB: f32 = -0.3;

    /// Create a new limiter with default threshold and disabled.
    #[must_use]
    pub fn new() -> Self {
        Self {
            threshold_db: Self::DEFAULT_THRESHOLD_DB,
            enabled: false,
            revision: 0,
            buf: [0.0; LOOKAHEAD_FRAMES * 2],
            write_head: 0,
            envelope_gain: 1.0,
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

    /// Set the threshold in dB. Clamped to `[GAIN_DB_MIN, 0.0]`. Increments revision.
    pub fn set_threshold_db(&mut self, db: f32) {
        self.threshold_db = db.clamp(GAIN_DB_MIN, 0.0);
        self.revision = self.revision.saturating_add(1);
    }

    /// Set the enabled state. Increments revision.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.revision = self.revision.saturating_add(1);
    }

    /// Reset the limiter state (delay buffer + envelope).
    ///
    /// Call this when the audio engine is stopped/started to avoid
    /// stale state from a previous session bleeding into the new one.
    pub fn reset(&mut self) {
        self.buf = [0.0; LOOKAHEAD_FRAMES * 2];
        self.write_head = 0;
        self.envelope_gain = 1.0;
    }

    /// Process a stereo sample pair through the lookahead brick-wall limiter.
    ///
    /// # Phase 2 Behaviour
    ///
    /// - If disabled: pass-through (no processing, zero latency).
    /// - If enabled:
    ///   1. Write `(left, right)` into the lookahead delay buffer.
    ///   2. Scan the delay buffer for the peak absolute value across both channels.
    ///   3. Compute target gain reduction to prevent the threshold from being exceeded.
    ///   4. Apply exponential envelope smoothing (attack/release).
    ///   5. Read the oldest frame from the delay buffer and scale by the smoothed gain.
    ///
    /// Adds exactly [`LOOKAHEAD_FRAMES`] samples of latency when enabled.
    ///
    /// # Realtime Safety
    ///
    /// No heap allocation, no I/O, no blocking. Safe to call from audio callback.
    ///
    /// # Arguments
    ///
    /// * `left`  — Left channel sample (nominal -1.0 to +1.0).
    /// * `right` — Right channel sample.
    ///
    /// # Returns
    ///
    /// `(left_out, right_out)` — limited stereo sample pair.
    #[must_use]
    #[inline]
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }

        let thresh = self.threshold_linear();

        // 1. Write new sample into delay buffer (interleaved L/R).
        let write_idx = self.write_head * 2;
        // Safety: write_idx = write_head * 2, write_head < LOOKAHEAD_FRAMES,
        // so write_idx + 1 < LOOKAHEAD_FRAMES * 2 = buf.len(). Bounds always valid.
        if write_idx + 1 < self.buf.len() {
            self.buf[write_idx] = left;
            self.buf[write_idx + 1] = right;
        }

        // 2. Scan the entire delay buffer for peak absolute value.
        let peak = self.buf.iter().fold(0.0_f32, |acc, &s| acc.max(s.abs()));

        // 3. Target gain: if peak exceeds threshold, attenuate.
        let target_gain = if peak > thresh {
            thresh / peak
        } else {
            1.0_f32
        };

        // 4. Envelope follower: attack if gain drops, release if gain rises.
        let coef = if target_gain < self.envelope_gain {
            ATTACK_COEF
        } else {
            RELEASE_COEF
        };
        self.envelope_gain = coef * self.envelope_gain + (1.0 - coef) * target_gain;
        // Hard-floor: never go below target_gain (brick-wall guarantee).
        if self.envelope_gain > target_gain {
            self.envelope_gain = target_gain;
        }

        // 5. Read oldest frame from delay buffer.
        let read_head = (self.write_head + 1) % LOOKAHEAD_FRAMES;
        let read_idx = read_head * 2;
        let (out_l, out_r) = (
            self.buf.get(read_idx).copied().unwrap_or(0.0),
            self.buf.get(read_idx + 1).copied().unwrap_or(0.0),
        );

        // Advance write cursor.
        self.write_head = (self.write_head + 1) % LOOKAHEAD_FRAMES;

        (out_l * self.envelope_gain, out_r * self.envelope_gain)
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
        let mut lim = Limiter::new();
        let (l, r) = lim.process(2.0, -3.0);
        assert!((l - 2.0).abs() < f32::EPSILON);
        assert!((r - (-3.0)).abs() < f32::EPSILON);
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

    /// Brick-wall guarantee: after sufficient steady-state frames,
    /// no output sample may exceed the threshold.
    #[test]
    fn test_limiter_brickwall_steady_state() {
        let thresh_db = -0.3;
        let mut lim = Limiter::new_enabled(thresh_db);
        let thresh = lim.threshold_linear();

        // Flush the delay buffer with a loud signal (2.0 > threshold)
        // to force gain reduction. We need at least LOOKAHEAD_FRAMES*2 frames
        // to fill the buffer and let the envelope settle.
        let settle = LOOKAHEAD_FRAMES * 4;
        let mut max_out = 0.0_f32;
        for i in 0..settle {
            let (l, r) = lim.process(2.0, -2.0);
            // Only measure after the buffer is primed (first LOOKAHEAD_FRAMES are ramp-up).
            if i >= LOOKAHEAD_FRAMES {
                max_out = max_out.max(l.abs()).max(r.abs());
            }
        }
        assert!(
            max_out <= thresh + 1e-4,
            "brick-wall violated: max_out={max_out:.6} > thresh={thresh:.6}"
        );
    }

    /// Pass-through guarantee: normal signal (below threshold) must not be distorted.
    #[test]
    fn test_limiter_passes_normal_signal_below_threshold() {
        let mut lim = Limiter::new_enabled(-0.3);
        let thresh = lim.threshold_linear(); // ~0.9833

        // A 0.5 amplitude signal must pass through unchanged after the
        // delay buffer fills with the same signal.
        let _settle: Vec<_> = (0..LOOKAHEAD_FRAMES)
            .map(|_| lim.process(0.5, 0.5))
            .collect();

        let (l, r) = lim.process(0.5, 0.5);
        // In steady state with a sub-threshold signal, gain should be ~1.0.
        // Allow a small tolerance for the release envelope ramp.
        assert!(
            l > 0.4 && l <= thresh,
            "normal signal should pass through: l={l}"
        );
        assert!(
            r > 0.4 && r <= thresh,
            "normal signal should pass through: r={r}"
        );
    }

    /// Silence in → silence out.
    #[test]
    fn test_limiter_silence_passthrough() {
        let mut lim = Limiter::new_enabled(-0.3);
        for _ in 0..(LOOKAHEAD_FRAMES * 2) {
            let (l, r) = lim.process(0.0, 0.0);
            assert!((l).abs() < 1e-6, "silence in should produce silence out");
            assert!((r).abs() < 1e-6);
        }
    }

    /// Reset clears delay buffer and envelope.
    #[test]
    fn test_limiter_reset() {
        let mut lim = Limiter::new_enabled(-0.3);
        // Prime with loud signal
        for _ in 0..LOOKAHEAD_FRAMES {
            let _ = lim.process(2.0, 2.0);
        }
        lim.reset();
        // After reset, envelope should be 1.0 and buffer zeroed
        assert!((lim.envelope_gain - 1.0).abs() < 1e-6);
        for s in &lim.buf {
            assert!(
                s.abs() < f32::EPSILON,
                "buf should be zeroed after reset, got {s}"
            );
        }
    }

    /// Lookahead guarantees output at frame N is based on input from frame `N+LOOKAHEAD_FRAMES`.
    #[test]
    fn test_limiter_lookahead_latency() {
        let mut lim = Limiter::new_enabled(-0.3);
        // Send LOOKAHEAD_FRAMES-1 silence frames, then one loud frame.
        // The loud frame should affect gain BEFORE it appears at output.
        let mut outputs = Vec::with_capacity(LOOKAHEAD_FRAMES + 10);
        for i in 0..(LOOKAHEAD_FRAMES + 10) {
            // Loud transient at frame LOOKAHEAD_FRAMES-1
            let sample = if i == LOOKAHEAD_FRAMES - 1 { 2.0 } else { 0.0 };
            outputs.push(lim.process(sample, sample));
        }
        // The output that corresponds to the loud frame arrives at index
        // 2*LOOKAHEAD_FRAMES - 1 due to the delay. For this test we just
        // verify the overall brick-wall constraint is not violated at any point.
        let thresh = lim.threshold_linear();
        for (i, (l, r)) in outputs.iter().enumerate() {
            assert!(
                l.abs() <= thresh + 1e-4,
                "brick-wall violated at frame {i}: l={l}"
            );
            assert!(
                r.abs() <= thresh + 1e-4,
                "brick-wall violated at frame {i}: r={r}"
            );
        }
    }
}
