//! `MixEngine` — top-level coordinator of channels and mixes.
//!
//! The `MixEngine` owns all `Channel`s and `Mix`es. It provides the
//! single entry point for processing one audio frame across all mixes.
//!
//! # Realtime Safety
//!
//! `process_frame` is realtime-safe: fixed-size stack allocation only,
//! no heap, no I/O, no blocking operations.
//!
//! State mutations (adding channels, changing gains) happen on the control
//! thread and must be applied atomically via a double-buffer or lock-free
//! message queue before the next audio callback. The infrastructure for
//! that handoff lives in the future `mix-engine-rt` crate.

use crate::{mix::MixError, Channel, Mix, MAX_CHANNELS, MAX_MIXES};

/// The mix engine: owns all channels and mixes for a session.
///
/// Fixed capacity: [`MAX_CHANNELS`] channels and [`MAX_MIXES`] mixes.
/// No heap allocation in the audio path.
#[derive(Debug, Clone)]
pub struct MixEngine {
    /// All input channels. `None` = slot unused.
    channels: [Option<Channel>; MAX_CHANNELS],

    /// All output mixes. `None` = slot unused.
    mixes: [Option<Mix>; MAX_MIXES],

    /// Monotonic revision counter for the engine state.
    revision: u64,
}

/// Output of one `process_frame` call.
///
/// Fixed-size: one stereo pair per mix slot.
/// Unoccupied mix slots output `(0.0, 0.0)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameOutput {
    /// Stereo output pairs indexed by mix slot.
    pub mixes: [(f32, f32); MAX_MIXES],
}

impl FrameOutput {
    /// All-zero (silence) frame output.
    #[must_use]
    pub const fn silence() -> Self {
        Self {
            mixes: [(0.0, 0.0); MAX_MIXES],
        }
    }
}

impl MixEngine {
    /// Create a new empty mix engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            channels: core::array::from_fn(|_| None),
            mixes: core::array::from_fn(|_| None),
            revision: 0,
        }
    }

    /// Returns the current engine revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Add or replace a channel at the given slot index.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::ChannelOutOfRange` if `index >= MAX_CHANNELS`.
    pub fn set_channel(&mut self, index: usize, channel: Channel) -> Result<(), EngineError> {
        if index >= MAX_CHANNELS {
            return Err(EngineError::ChannelOutOfRange {
                index,
                max: MAX_CHANNELS,
            });
        }
        if let Some(slot) = self.channels.get_mut(index) {
            *slot = Some(channel);
        }
        self.revision = self.revision.saturating_add(1);
        Ok(())
    }

    /// Remove a channel at the given slot index.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::ChannelOutOfRange` if `index >= MAX_CHANNELS`.
    pub fn remove_channel(&mut self, index: usize) -> Result<(), EngineError> {
        if index >= MAX_CHANNELS {
            return Err(EngineError::ChannelOutOfRange {
                index,
                max: MAX_CHANNELS,
            });
        }
        if let Some(slot) = self.channels.get_mut(index) {
            *slot = None;
        }
        self.revision = self.revision.saturating_add(1);
        Ok(())
    }

    /// Returns a reference to the channel at the given index.
    #[must_use]
    pub fn channel(&self, index: usize) -> Option<&Channel> {
        self.channels.get(index)?.as_ref()
    }

    /// Add or replace a mix at the given slot index.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::MixOutOfRange` if `index >= MAX_MIXES`.
    pub fn set_mix(&mut self, index: usize, mix: Mix) -> Result<(), EngineError> {
        if index >= MAX_MIXES {
            return Err(EngineError::MixOutOfRange {
                index,
                max: MAX_MIXES,
            });
        }
        if let Some(slot) = self.mixes.get_mut(index) {
            *slot = Some(mix);
        }
        self.revision = self.revision.saturating_add(1);
        Ok(())
    }

    /// Remove a mix at the given slot index.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::MixOutOfRange` if `index >= MAX_MIXES`.
    pub fn remove_mix(&mut self, index: usize) -> Result<(), EngineError> {
        if index >= MAX_MIXES {
            return Err(EngineError::MixOutOfRange {
                index,
                max: MAX_MIXES,
            });
        }
        if let Some(slot) = self.mixes.get_mut(index) {
            *slot = None;
        }
        self.revision = self.revision.saturating_add(1);
        Ok(())
    }

    /// Returns a reference to the mix at the given index.
    #[must_use]
    pub fn mix(&self, index: usize) -> Option<&Mix> {
        self.mixes.get(index)?.as_ref()
    }

    /// Returns a mutable reference to the mix at the given index.
    pub fn mix_mut(&mut self, index: usize) -> Option<&mut Mix> {
        self.mixes.get_mut(index)?.as_mut()
    }

    /// Process one audio frame across all mixes.
    ///
    /// # Arguments
    ///
    /// * `input_samples` — slice of mono samples, one per channel slot.
    ///   Length must be ≤ `MAX_CHANNELS`. Missing slots → 0.0.
    ///
    /// # Returns
    ///
    /// [`FrameOutput`] with stereo pairs for each mix slot.
    ///
    /// # Realtime Safety
    ///
    /// - No heap allocation.
    /// - No I/O.
    /// - No blocking.
    /// - Fixed iteration over fixed-size arrays.
    #[must_use]
    pub fn process_frame(&self, input_samples: &[f32]) -> FrameOutput {
        // Build a flat channel slice for mix processing (stack-allocated)
        // We need a slice of `Channel` for borrow-check reasons; we build
        // a compact view from the sparse `Option<Channel>` array.
        //
        // To avoid heap allocation we process sends inline per mix.
        // The `Mix::process` already handles None/missing via index bounds.

        // Provide a flat view: for each slot, use a dummy sentinel if None.
        // We pass the full channels array as a slice of references via
        // a local stack buffer of Option references.
        let mut out = FrameOutput::silence();

        // Build a compact channel slice: use a fixed-size stack array.
        // Slots without channels use a sentinel Channel (muted, 0 dB).
        // IMPORTANT: no allocation — fixed array on stack.
        let channel_slice: [Channel; MAX_CHANNELS] = core::array::from_fn(|i| {
            self.channels
                .get(i)
                .and_then(Option::as_ref)
                .cloned()
                .unwrap_or_else(|| {
                    let mut sentinel = Channel::new(i as u32, "");
                    sentinel.set_muted(true);
                    sentinel
                })
        });

        for (mix_idx, mix_opt) in self.mixes.iter().enumerate() {
            if let Some(mix) = mix_opt {
                let (l, r) = mix.process(input_samples, &channel_slice);
                if let Some(slot) = out.mixes.get_mut(mix_idx) {
                    *slot = (l, r);
                }
            }
        }

        out
    }

    /// Returns the number of active (non-None) channel slots.
    #[must_use]
    pub fn active_channel_count(&self) -> usize {
        self.channels.iter().filter(|c| c.is_some()).count()
    }

    /// Returns the number of active (non-None) mix slots.
    #[must_use]
    pub fn active_mix_count(&self) -> usize {
        self.mixes.iter().filter(|m| m.is_some()).count()
    }
}

impl Default for MixEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors from `MixEngine` operations.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    /// Channel index out of bounds.
    ChannelOutOfRange {
        /// Requested index.
        index: usize,
        /// Maximum allowed (exclusive).
        max: usize,
    },
    /// Mix index out of bounds.
    MixOutOfRange {
        /// Requested index.
        index: usize,
        /// Maximum allowed (exclusive).
        max: usize,
    },
    /// Propagated mix error.
    Mix(MixError),
}

impl From<MixError> for EngineError {
    fn from(e: MixError) -> Self {
        Self::Mix(e)
    }
}

impl core::fmt::Display for EngineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ChannelOutOfRange { index, max } => {
                write!(f, "channel index {index} out of range (max {max})")
            }
            Self::MixOutOfRange { index, max } => {
                write!(f, "mix index {index} out of range (max {max})")
            }
            Self::Mix(e) => write!(f, "mix error: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mix_send::MixSend;

    fn make_engine_with_one_send() -> MixEngine {
        let mut engine = MixEngine::new();
        let ch = Channel::new(0, "CH01");
        engine.set_channel(0, ch).expect("set_channel");

        let mut mix = Mix::new(1, "Monitor 1");
        let send = MixSend::new(0, 1);
        mix.set_send(0, send).expect("set_send");
        engine.set_mix(0, mix).expect("set_mix");

        engine
    }

    #[test]
    fn test_engine_defaults() {
        let engine = MixEngine::new();
        assert_eq!(engine.active_channel_count(), 0);
        assert_eq!(engine.active_mix_count(), 0);
        assert_eq!(engine.revision(), 0);
    }

    #[test]
    fn test_engine_set_channel() {
        let mut engine = MixEngine::new();
        let ch = Channel::new(0, "VOC");
        engine.set_channel(0, ch).expect("ok");
        assert_eq!(engine.active_channel_count(), 1);
        assert!(engine.channel(0).is_some());
    }

    #[test]
    fn test_engine_channel_out_of_range() {
        let mut engine = MixEngine::new();
        let ch = Channel::new(0, "x");
        assert!(engine.set_channel(MAX_CHANNELS, ch).is_err());
    }

    #[test]
    fn test_engine_set_mix() {
        let mut engine = MixEngine::new();
        let mix = Mix::new(1, "M1");
        engine.set_mix(0, mix).expect("ok");
        assert_eq!(engine.active_mix_count(), 1);
    }

    #[test]
    fn test_engine_mix_out_of_range() {
        let mut engine = MixEngine::new();
        let mix = Mix::new(1, "x");
        assert!(engine.set_mix(MAX_MIXES, mix).is_err());
    }

    #[test]
    fn test_engine_process_empty_returns_silence() {
        let engine = MixEngine::new();
        let out = engine.process_frame(&[0.5; MAX_CHANNELS]);
        for (l, r) in out.mixes {
            assert!((l).abs() < f32::EPSILON);
            assert!((r).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_engine_process_with_active_send() {
        let engine = make_engine_with_one_send();
        let samples = [1.0_f32; MAX_CHANNELS];
        let out = engine.process_frame(&samples);
        // Mix 0 should have output (centre pan → l ≈ r ≈ 0.707)
        let (l, r) = out.mixes[0];
        assert!(l > 0.0, "mix 0 left should be non-zero");
        assert!(r > 0.0, "mix 0 right should be non-zero");
        assert!((l - r).abs() < 1e-5, "centre pan should be L=R");
        // Mix 1 (no mix in slot 1) → silence
        let (l1, r1) = out.mixes[1];
        assert!((l1).abs() < f32::EPSILON);
        assert!((r1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_engine_remove_channel() {
        let mut engine = make_engine_with_one_send();
        engine.remove_channel(0).expect("ok");
        // Channel removed → its slot is now a muted sentinel
        // Mix still has the send but channel is muted → silence
        let samples = [1.0_f32; MAX_CHANNELS];
        let out = engine.process_frame(&samples);
        let (l, r) = out.mixes[0];
        assert!((l).abs() < f32::EPSILON, "removed channel should be silent");
        assert!((r).abs() < f32::EPSILON);
    }

    #[test]
    fn test_engine_remove_mix() {
        let mut engine = make_engine_with_one_send();
        engine.remove_mix(0).expect("ok");
        let samples = [1.0_f32; MAX_CHANNELS];
        let out = engine.process_frame(&samples);
        let (l, r) = out.mixes[0];
        assert!((l).abs() < f32::EPSILON);
        assert!((r).abs() < f32::EPSILON);
    }

    #[test]
    fn test_engine_revision_increments() {
        let mut engine = MixEngine::new();
        assert_eq!(engine.revision(), 0);
        let ch = Channel::new(0, "x");
        engine.set_channel(0, ch).expect("ok");
        assert_eq!(engine.revision(), 1);
        let mix = Mix::new(1, "m");
        engine.set_mix(0, mix).expect("ok");
        assert_eq!(engine.revision(), 2);
        engine.remove_channel(0).expect("ok");
        assert_eq!(engine.revision(), 3);
        engine.remove_mix(0).expect("ok");
        assert_eq!(engine.revision(), 4);
    }

    #[test]
    fn test_frame_output_silence() {
        let out = FrameOutput::silence();
        for (l, r) in out.mixes {
            assert!((l).abs() < f32::EPSILON);
            assert!((r).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_engine_two_mixes_independent() {
        let mut engine = MixEngine::new();
        let ch = Channel::new(0, "CH01");
        engine.set_channel(0, ch).expect("ok");

        // Mix 0: ch0 at 0dB, centre
        let mut mix0 = Mix::new(1, "Mix A");
        mix0.set_send(0, MixSend::new(0, 1)).expect("ok");

        // Mix 1: ch0 at -6dB, centre
        let mut mix1 = Mix::new(2, "Mix B");
        let mut send1 = MixSend::new(0, 2);
        send1.set_gain_db(-6.0);
        mix1.set_send(0, send1).expect("ok");

        engine.set_mix(0, mix0).expect("ok");
        engine.set_mix(1, mix1).expect("ok");

        let out = engine.process_frame(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let (l0, _r0) = out.mixes[0];
        let (l1, _r1) = out.mixes[1];

        // Mix 1 should be ~0.5x mix 0 (−6 dB ≈ ×0.501)
        let ratio = l1 / l0;
        assert!(
            (ratio - 0.501).abs() < 0.01,
            "Mix 1 should be ~-6dB vs Mix 0, got ratio={ratio:.4}"
        );
    }

    #[test]
    fn test_engine_error_display() {
        let err = EngineError::ChannelOutOfRange { index: 10, max: 8 };
        assert!(err.to_string().contains("10"));
        let err2 = EngineError::MixOutOfRange { index: 5, max: 2 };
        assert!(err2.to_string().contains('5'));
    }
}
