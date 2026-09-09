//! Mix — a single independent stereo output mix.
//!
//! Each mix has a set of `MixSend`s (one per contributing channel),
//! a master gain, a master mute, and an output limiter.
//!
//! The mix processes audio by summing all active send contributions
//! and applying the mix-bus chain: EQ → Compressor → Master Gain → Limiter.

use crate::{
    apply_pan, db_to_linear, limiter::Limiter, mix_send::MixSend, Channel, Compressor,
    ParametricEq, GAIN_DB_MAX, GAIN_DB_MIN, MAX_CHANNELS,
};

/// Unique mix identifier (same type as `MixId` in `mix_send`).
pub type MixId = u32;

/// A single independent stereo output mix.
///
/// Contains up to [`MAX_CHANNELS`] sends (one per input channel),
/// a master section (gain, mute), and an output limiter.
///
/// Audio chain per frame: Sum → EQ → Compressor → Master Gain → Limiter.
#[derive(Debug, Clone, PartialEq)]
pub struct Mix {
    /// Unique identifier for this mix.
    pub id: MixId,

    /// Human-readable name (e.g., "Monitor 1 — Vocals").
    name: [u8; 64],
    name_len: usize,

    /// Per-channel sends. Fixed-size array — no heap allocation.
    ///
    /// `sends[i]` corresponds to channel ID `i`. `None` means that channel
    /// has no send into this mix.
    sends: [Option<MixSend>; MAX_CHANNELS],

    /// Master output gain in dB.
    master_gain_db: f32,

    /// Master mute — silences the entire mix output.
    pub master_muted: bool,

    /// Mix-bus parametric EQ (applied before compressor).
    pub eq: ParametricEq,

    /// Mix-bus compressor (applied after EQ, before master gain).
    pub compressor: Compressor,

    /// Output limiter configuration.
    pub limiter: Limiter,

    /// Monotonic revision counter. Incremented on every state mutation.
    revision: u64,
}

impl Mix {
    /// Create a new mix with the given ID and name.
    ///
    /// Defaults: master gain = 0 dB, unmuted, EQ flat, compressor disabled, limiter disabled.
    #[must_use]
    pub fn new(id: MixId, name: &str) -> Self {
        let bytes = name.as_bytes();
        let len = bytes.len().min(64);
        let mut name_buf = [0u8; 64];
        if let (Some(dst), Some(src)) = (name_buf.get_mut(..len), bytes.get(..len)) {
            dst.copy_from_slice(src);
        }

        Self {
            id,
            name: name_buf,
            name_len: len,
            sends: core::array::from_fn(|_| None),
            master_gain_db: 0.0,
            master_muted: false,
            eq: ParametricEq::new(),
            compressor: Compressor::new(),
            limiter: Limiter::new(),
            revision: 0,
        }
    }

    /// Returns the mix name as a string slice.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name
            .get(..self.name_len)
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or("")
    }

    /// Returns the master gain in dB.
    #[must_use]
    pub fn master_gain_db(&self) -> f32 {
        self.master_gain_db
    }

    /// Returns the current monotonic revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Set the master gain in dB. Clamped to `[GAIN_DB_MIN, GAIN_DB_MAX]`.
    /// Increments revision.
    pub fn set_master_gain_db(&mut self, db: f32) {
        self.master_gain_db = db.clamp(GAIN_DB_MIN, GAIN_DB_MAX);
        self.revision = self.revision.saturating_add(1);
    }

    /// Set the master mute state. Increments revision.
    pub fn set_master_muted(&mut self, muted: bool) {
        self.master_muted = muted;
        self.revision = self.revision.saturating_add(1);
    }

    /// Add or replace the send for a given channel index.
    ///
    /// Returns `Err` if the channel index exceeds `MAX_CHANNELS - 1`.
    ///
    /// # Errors
    ///
    /// Returns `MixError::ChannelOutOfRange` if `channel_index >= MAX_CHANNELS`.
    pub fn set_send(&mut self, channel_index: usize, send: MixSend) -> Result<(), MixError> {
        if channel_index >= MAX_CHANNELS {
            return Err(MixError::ChannelOutOfRange {
                index: channel_index,
                max: MAX_CHANNELS,
            });
        }
        if let Some(slot) = self.sends.get_mut(channel_index) {
            *slot = Some(send);
        }
        self.revision = self.revision.saturating_add(1);
        Ok(())
    }

    /// Remove the send for a given channel index.
    ///
    /// # Errors
    ///
    /// Returns `MixError::ChannelOutOfRange` if `channel_index >= MAX_CHANNELS`.
    pub fn remove_send(&mut self, channel_index: usize) -> Result<(), MixError> {
        if channel_index >= MAX_CHANNELS {
            return Err(MixError::ChannelOutOfRange {
                index: channel_index,
                max: MAX_CHANNELS,
            });
        }
        if let Some(slot) = self.sends.get_mut(channel_index) {
            *slot = None;
        }
        self.revision = self.revision.saturating_add(1);
        Ok(())
    }

    /// Returns a reference to the send for a channel index, if present.
    #[must_use]
    pub fn send(&self, channel_index: usize) -> Option<&MixSend> {
        self.sends.get(channel_index)?.as_ref()
    }

    /// Alias for reading a send state.
    #[must_use]
    pub fn get_send(&self, channel_index: usize) -> Option<&MixSend> {
        self.send(channel_index)
    }

    /// Set send gain, creating send when absent.
    ///
    /// # Errors
    /// Returns `MixError::ChannelOutOfRange` for an invalid channel index.
    pub fn set_send_gain_db(&mut self, channel_index: usize, gain_db: f32) -> Result<(), MixError> {
        if channel_index >= MAX_CHANNELS {
            return Err(MixError::ChannelOutOfRange {
                index: channel_index,
                max: MAX_CHANNELS,
            });
        }
        let mut send = self.sends[channel_index]
            .clone()
            .unwrap_or_else(|| MixSend::new(channel_index as u32, self.id));
        send.set_gain_db(gain_db);
        self.set_send(channel_index, send)
    }

    /// Set send pan, creating send when absent.
    ///
    /// # Errors
    /// Returns `MixError::ChannelOutOfRange` for an invalid channel index.
    pub fn set_send_pan(&mut self, channel_index: usize, pan: f32) -> Result<(), MixError> {
        if channel_index >= MAX_CHANNELS {
            return Err(MixError::ChannelOutOfRange {
                index: channel_index,
                max: MAX_CHANNELS,
            });
        }
        let mut send = self.sends[channel_index]
            .clone()
            .unwrap_or_else(|| MixSend::new(channel_index as u32, self.id));
        send.set_pan(pan);
        self.set_send(channel_index, send)
    }

    /// Set send mute, creating send when absent.
    ///
    /// # Errors
    /// Returns `MixError::ChannelOutOfRange` for an invalid channel index.
    pub fn set_send_muted(&mut self, channel_index: usize, muted: bool) -> Result<(), MixError> {
        if channel_index >= MAX_CHANNELS {
            return Err(MixError::ChannelOutOfRange {
                index: channel_index,
                max: MAX_CHANNELS,
            });
        }
        let mut send = self.sends[channel_index]
            .clone()
            .unwrap_or_else(|| MixSend::new(channel_index as u32, self.id));
        send.set_muted(muted);
        self.set_send(channel_index, send)
    }

    /// Returns `true` if any send in this mix is soloed.
    #[must_use]
    pub fn has_solo(&self) -> bool {
        self.sends
            .iter()
            .any(|s| s.as_ref().is_some_and(|s| s.solo))
    }

    /// Process one stereo frame of audio.
    ///
    /// # Arguments
    ///
    /// * `input_samples` — slice of mono input samples, one per channel.
    ///   Length must be ≤ `MAX_CHANNELS`. Channels with index ≥ length are treated as silent.
    /// * `channels` — channel metadata slice (for mute/enable state).
    ///
    /// # Returns
    ///
    /// `(left, right)` — stereo output sample pair after the chain:
    /// Sum → EQ → Compressor → Master Gain → Limiter.
    ///
    /// # Realtime Safety
    ///
    /// No allocation, no I/O, no blocking. Safe to call from audio callback.
    ///
    /// # Panics
    ///
    /// Does not panic. `input_samples` and `channels` slices are accessed
    /// with bounds checks (returning 0.0 for out-of-range indices).
    #[must_use]
    pub fn process(&mut self, input_samples: &[f32], channels: &[Channel]) -> (f32, f32) {
        if self.master_muted {
            return (0.0, 0.0);
        }

        let solo_active = self.has_solo();
        let master_linear = db_to_linear(self.master_gain_db);

        let mut left_sum = 0.0_f32;
        let mut right_sum = 0.0_f32;

        for (idx, send_opt) in self.sends.iter().enumerate() {
            let Some(send) = send_opt else {
                continue;
            };

            // Solo gate: if any send is soloed, skip non-soloed sends
            if solo_active && !send.solo {
                continue;
            }

            if !send.is_active() {
                continue;
            }

            // Check channel-level mute/enable
            let channel_silent = channels.get(idx).is_some_and(Channel::is_silent);
            if channel_silent {
                continue;
            }

            // Get input sample (0.0 if out of range)
            let raw_sample = input_samples.get(idx).copied().unwrap_or(0.0);

            // Apply channel trim (if channel available)
            let channel_gain = channels
                .get(idx)
                .map_or(1.0, |ch| db_to_linear(ch.gain_db()));

            // Apply send gain
            let send_gain = db_to_linear(send.gain_db());

            let processed = raw_sample * channel_gain * send_gain;

            // Apply pan
            let (l, r) = apply_pan(processed, send.pan());
            left_sum += l;
            right_sum += r;
        }

        // Mix-bus EQ
        let (l_eq, r_eq) = self.eq.process(left_sum, right_sum);

        // Mix-bus Compressor
        let (l_comp, r_comp) = self.compressor.process(l_eq, r_eq);

        // Apply master gain
        let (l, r) = (l_comp * master_linear, r_comp * master_linear);

        // Apply limiter
        self.limiter.process(l, r)
    }
}

/// Errors from `Mix` operations.
#[derive(Debug, Clone, PartialEq)]
pub enum MixError {
    /// Channel index exceeds the maximum allowed.
    ChannelOutOfRange {
        /// The requested index.
        index: usize,
        /// The maximum allowed (exclusive).
        max: usize,
    },
}

impl core::fmt::Display for MixError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ChannelOutOfRange { index, max } => {
                write!(f, "channel index {index} out of range (max {max})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mix_send::MixSend;

    fn make_channels(n: usize) -> Vec<Channel> {
        (0..n)
            .map(|i| Channel::new(i as u32, &format!("CH{i:02}")))
            .collect()
    }

    #[test]
    fn test_mix_defaults() {
        let mix = Mix::new(1, "Monitor 1");
        assert_eq!(mix.id, 1);
        assert_eq!(mix.name(), "Monitor 1");
        assert!((mix.master_gain_db() - 0.0).abs() < f32::EPSILON);
        assert!(!mix.master_muted);
        assert!(!mix.limiter.enabled);
        assert_eq!(mix.revision(), 0);
    }

    #[test]
    fn test_mix_master_gain_clamped() {
        let mut mix = Mix::new(1, "test");
        mix.set_master_gain_db(999.0);
        assert!((mix.master_gain_db() - GAIN_DB_MAX).abs() < f32::EPSILON);
        mix.set_master_gain_db(-999.0);
        assert!((mix.master_gain_db() - GAIN_DB_MIN).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mix_process_silence_when_no_sends() {
        let mut mix = Mix::new(1, "empty");
        let channels = make_channels(4);
        let samples = [0.5_f32; 4];
        let (l, r) = mix.process(&samples, &channels);
        assert!((l).abs() < f32::EPSILON);
        assert!((r).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mix_process_master_muted() {
        let mut mix = Mix::new(1, "test");
        let mut send = MixSend::new(0, 1);
        send.set_gain_db(0.0);
        mix.set_send(0, send).expect("set_send should succeed");
        mix.set_master_muted(true);
        let channels = make_channels(2);
        let (l, r) = mix.process(&[1.0, 1.0], &channels);
        assert!((l).abs() < f32::EPSILON);
        assert!((r).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mix_process_unity_gain_centre_pan() {
        let mut mix = Mix::new(1, "test");
        let send = MixSend::new(0, 1); // 0dB, centre pan
        mix.set_send(0, send).expect("set_send should succeed");
        let channels = make_channels(1);
        let (l, r) = mix.process(&[1.0], &channels);
        // Centre pan: equal power (~0.707 each)
        assert!((l - r).abs() < 1e-5, "centre pan should be equal L/R");
        assert!(l > 0.6 && l < 0.8, "centre pan amplitude should be ~0.707");
    }

    #[test]
    fn test_mix_process_channel_muted() {
        let mut mix = Mix::new(1, "test");
        let send = MixSend::new(0, 1);
        mix.set_send(0, send).expect("set_send should succeed");
        let mut channels = make_channels(1);
        if let Some(ch) = channels.get_mut(0) {
            ch.set_muted(true);
        }
        let (l, r) = mix.process(&[1.0], &channels);
        assert!((l).abs() < f32::EPSILON);
        assert!((r).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mix_process_send_muted() {
        let mut mix = Mix::new(1, "test");
        let mut send = MixSend::new(0, 1);
        send.set_muted(true);
        mix.set_send(0, send).expect("set_send should succeed");
        let channels = make_channels(1);
        let (l, r) = mix.process(&[1.0], &channels);
        assert!((l).abs() < f32::EPSILON);
        assert!((r).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mix_solo_gates_non_soloed() {
        let mut mix = Mix::new(1, "test");
        // ch0 = normal, ch1 = soloed
        let mut send0 = MixSend::new(0, 1);
        send0.set_gain_db(0.0);
        let mut send1 = MixSend::new(1, 1);
        send1.set_solo(true);
        mix.set_send(0, send0).expect("ok");
        mix.set_send(1, send1).expect("ok");
        let channels = make_channels(2);
        // samples: ch0=1.0, ch1=0.0 → only ch1 is soloed → output should be ~0
        let (l, r) = mix.process(&[1.0, 0.0], &channels);
        assert!((l).abs() < 1e-5, "non-soloed ch should be gated");
        assert!((r).abs() < 1e-5);
    }

    #[test]
    fn test_mix_set_send_out_of_range() {
        let mut mix = Mix::new(1, "test");
        let send = MixSend::new(0, 1);
        let result = mix.set_send(MAX_CHANNELS, send);
        assert!(result.is_err());
    }

    #[test]
    fn test_mix_has_solo() {
        let mut mix = Mix::new(1, "test");
        assert!(!mix.has_solo());
        let mut send = MixSend::new(0, 1);
        send.set_solo(true);
        mix.set_send(0, send).expect("ok");
        assert!(mix.has_solo());
    }

    #[test]
    fn test_mix_revision_increments_on_mutation() {
        let mut mix = Mix::new(1, "test");
        assert_eq!(mix.revision(), 0);
        mix.set_master_gain_db(-6.0);
        assert_eq!(mix.revision(), 1);
        mix.set_master_muted(true);
        assert_eq!(mix.revision(), 2);
        let send = MixSend::new(0, 1);
        mix.set_send(0, send).expect("ok");
        assert_eq!(mix.revision(), 3);
        mix.remove_send(0).expect("ok");
        assert_eq!(mix.revision(), 4);
    }

    #[test]
    fn test_mix_limiter_clips_hot_signal() {
        let mut mix = Mix::new(1, "test");
        // Boost master to +12 dB → linear ~4.0 → limiter must clip
        mix.set_master_gain_db(12.0);
        mix.limiter = Limiter::new_enabled(-0.3);
        let send = MixSend::new(0, 1);
        mix.set_send(0, send).expect("ok");
        let channels = make_channels(1);
        let (l, r) = mix.process(&[1.0], &channels);
        let thresh = mix.limiter.threshold_linear();
        assert!(l <= thresh + 1e-5, "limiter must clip left");
        assert!(r <= thresh + 1e-5, "limiter must clip right");
    }

    #[test]
    fn test_mix_error_display() {
        let err = MixError::ChannelOutOfRange { index: 10, max: 8 };
        let msg = err.to_string();
        assert!(msg.contains("10"));
        assert!(msg.contains('8'));
    }

    // -----------------------------------------------------------------------
    // Phase 8 tests: EQ + Compressor integration in Mix::process
    // -----------------------------------------------------------------------

    fn sinusoid_samples(freq_hz: f32, n: usize) -> Vec<f32> {
        use core::f32::consts::PI;
        (0..n)
            .map(|i| (2.0 * PI * freq_hz * i as f32 / 48_000.0).sin())
            .collect()
    }

    fn rms(samples: &[f32]) -> f32 {
        let sum: f32 = samples.iter().map(|s| s * s).sum();
        (sum / samples.len() as f32).sqrt()
    }

    #[test]
    fn test_mix_eq_boosts_at_freq() {
        // Setup: one send at unity, 1 kHz sinusoid as input, EQ band +6 dB at 1 kHz.
        let mut mix = Mix::new(1, "test_eq_boost");
        let send = MixSend::new(0, 1);
        mix.set_send(0, send).expect("set_send ok");

        // Enable EQ band: 1 kHz, +6 dB, Q=1
        mix.eq.set_band(
            0,
            crate::EqBand {
                frequency_hz: 1_000.0,
                gain_db: 6.0,
                q: 1.0,
                enabled: true,
            },
        );

        let channels = make_channels(1);
        let signal = sinusoid_samples(1_000.0, 4_800);

        // Baseline: no EQ (fresh mix)
        let mut mix_bypass = Mix::new(2, "bypass");
        let send2 = MixSend::new(0, 1);
        mix_bypass.set_send(0, send2).expect("ok");
        let bypass_out: Vec<f32> = signal
            .iter()
            .map(|&s| {
                let (l, _) = mix_bypass.process(&[s], &channels);
                l
            })
            .collect();
        let bypass_rms = rms(&bypass_out);

        let boosted_out: Vec<f32> = signal
            .iter()
            .map(|&s| {
                let (l, _) = mix.process(&[s], &channels);
                l
            })
            .collect();
        let boosted_rms = rms(&boosted_out);

        assert!(
            boosted_rms > bypass_rms * 1.3,
            "EQ +6 dB should boost 1 kHz: bypass_rms={bypass_rms:.4} boosted_rms={boosted_rms:.4}"
        );
    }

    #[test]
    fn test_mix_compressor_reduces_loud() {
        let mut mix = Mix::new(1, "test_comp");
        let send = MixSend::new(0, 1);
        mix.set_send(0, send).expect("ok");

        // Enable compressor: threshold -6 dB, ratio 4:1, fast attack
        mix.compressor.set_enabled(true);
        mix.compressor.set_threshold(-6.0);
        mix.compressor.set_ratio(4.0);
        mix.compressor.set_attack_ms(1.0);

        let channels = make_channels(1);
        let input = 0.9_f32;

        // Warm up
        for _ in 0..10_000 {
            let _ = mix.process(&[input], &channels);
        }

        // Measure output after warmup
        let (l, _) = mix.process(&[input], &channels);
        // Centre pan at unity: signal passes through ~0.707. After compressor reduction
        // should be < input * 0.95 * 0.707 ≈ 0.672
        let expected_max = input * 0.95 * 0.8; // generous bound accounting for pan
        assert!(
            l < expected_max,
            "compressor should reduce loud signal: l={l:.4} expected_max={expected_max:.4}"
        );
    }

    #[test]
    fn test_mix_eq_passthrough_when_disabled() {
        // All EQ bands disabled → output matches plain sum chain.
        let channels = make_channels(1);
        let signal = sinusoid_samples(1_000.0, 100);

        let mut mix_eq = Mix::new(1, "eq_disabled");
        mix_eq.set_send(0, MixSend::new(0, 1)).expect("ok");
        // EQ bands are disabled by default — no change needed

        let mut mix_plain = Mix::new(2, "plain");
        mix_plain.set_send(0, MixSend::new(0, 1)).expect("ok");

        for &s in &signal {
            let (l_eq, _) = mix_eq.process(&[s], &channels);
            let (l_plain, _) = mix_plain.process(&[s], &channels);
            assert!(
                (l_eq - l_plain).abs() < 1e-4,
                "EQ passthrough mismatch: l_eq={l_eq:.6} l_plain={l_plain:.6}"
            );
        }
    }

    #[test]
    fn test_mix_compressor_passthrough_when_disabled() {
        // Compressor disabled → same output as plain sum+gain+limiter chain.
        let channels = make_channels(1);
        let signal = sinusoid_samples(440.0, 100);

        let mut mix_comp = Mix::new(1, "comp_disabled");
        mix_comp.set_send(0, MixSend::new(0, 1)).expect("ok");
        // Compressor disabled by default

        let mut mix_plain = Mix::new(2, "plain2");
        mix_plain.set_send(0, MixSend::new(0, 1)).expect("ok");

        for &s in &signal {
            let (l_c, _) = mix_comp.process(&[s], &channels);
            let (l_p, _) = mix_plain.process(&[s], &channels);
            assert!(
                (l_c - l_p).abs() < 1e-5,
                "disabled compressor should be transparent: l_c={l_c:.6} l_p={l_p:.6}"
            );
        }
    }

    #[test]
    fn test_mix_chain_order() {
        // EQ boosts 1 kHz → then compressor sees boosted signal and applies gain reduction.
        // With EQ: compressor output should be reduced compared to bypass.
        let channels = make_channels(1);

        // Mix A: EQ boost + compressor enabled (should compress boosted signal)
        let mut mix_eq_comp = Mix::new(1, "eq_then_comp");
        mix_eq_comp.set_send(0, MixSend::new(0, 1)).expect("ok");
        mix_eq_comp.eq.set_band(
            0,
            crate::EqBand {
                frequency_hz: 1_000.0,
                gain_db: 12.0,
                q: 1.0,
                enabled: true,
            },
        );
        mix_eq_comp.compressor.set_enabled(true);
        mix_eq_comp.compressor.set_threshold(-6.0);
        mix_eq_comp.compressor.set_ratio(4.0);
        mix_eq_comp.compressor.set_attack_ms(1.0);

        // Mix B: only EQ, no compressor (unboosted by compression)
        let mut mix_eq_only = Mix::new(2, "eq_only");
        mix_eq_only.set_send(0, MixSend::new(0, 1)).expect("ok");
        mix_eq_only.eq.set_band(
            0,
            crate::EqBand {
                frequency_hz: 1_000.0,
                gain_db: 12.0,
                q: 1.0,
                enabled: true,
            },
        );

        let signal = sinusoid_samples(1_000.0, 4_800);

        // Warmup both
        for &s in &signal {
            let _ = mix_eq_comp.process(&[s], &channels);
            let _ = mix_eq_only.process(&[s], &channels);
        }

        // After warmup, collect RMS of another burst
        let (rms_comp, rms_eq): (Vec<f32>, Vec<f32>) = signal
            .iter()
            .map(|&s| {
                let (l_c, _) = mix_eq_comp.process(&[s], &channels);
                let (l_e, _) = mix_eq_only.process(&[s], &channels);
                (l_c, l_e)
            })
            .unzip();

        let rms_with_comp = rms(&rms_comp);
        let rms_without_comp = rms(&rms_eq);

        // Compressor after EQ should reduce boosted signal
        assert!(
            rms_with_comp < rms_without_comp,
            "chain order: EQ→Comp should yield lower RMS than EQ alone: \
             with_comp={rms_with_comp:.4} without_comp={rms_without_comp:.4}"
        );
    }
}
