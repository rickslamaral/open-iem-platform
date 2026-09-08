//! Simulated audio backend — **SIMULATED**
//!
//! Runs the full DSP pipeline against a synthetic audio signal without
//! requiring any audio hardware or `PipeWire` installation. Suitable for:
//!
//! - VPS / CI builds where no JACK/`PipeWire` is available.
//! - Unit and integration tests.
//! - DSP algorithm validation without hardware.
//!
//! # Behaviour
//!
//! The simulated backend generates a fixed-amplitude sine wave at 440 Hz as
//! synthetic input for all channels, runs [`mix_engine::MixEngine::process_frame`]
//! for each frame, and collects the output into an internal buffer for inspection.
//!
//! It does **not** spawn an OS thread — `process_n_frames` is called synchronously
//! for deterministic testing. This mirrors the JACK callback model where the
//! backend calls our callback in its own realtime thread.
//!
//! # SIMULATED
//!
//! This backend does not connect to any real audio graph. All `process_frame`
//! calls happen in the calling thread (not a realtime thread). CPU timing
//! in `ProcessStats::cpu_us` is measured via `std::time::Instant`.

use std::sync::{Arc, Mutex};
use std::time::Instant;

#[allow(unused_imports)]
use mix_engine::{MixEngine, MAX_CHANNELS, MAX_MIXES, SAMPLE_RATE};

use crate::{
    backend::{Backend, BackendResult, ProcessStats},
    error::AudioEngineError,
};

/// Shared state between `SimulatedBackend` and the (synchronous) callback.
struct SimState {
    engine: MixEngine,
    /// Cumulative output samples per mix: `output_buffer[mix_idx]` = Vec of (L, R) pairs.
    output_buffer: [[Vec<(f32, f32)>; 1]; MAX_MIXES],
    #[allow(dead_code)]
    xrun_count: u64,
    frames_processed: u64,
}

/// **SIMULATED** — Audio backend that drives the mix engine with synthetic audio.
///
/// No hardware, no `PipeWire`, no threads. Use `process_n_frames` to run the pipeline.
pub struct SimulatedBackend {
    sample_rate: u32,
    buffer_frames: u32,
    active: bool,
    /// Phase accumulator for the 440 Hz test tone (radians, per channel).
    phase: [f32; MAX_CHANNELS],
    /// Shared DSP state (wrapped in Arc<Mutex> to mirror real thread handoff model).
    state: Arc<Mutex<SimState>>,
}

impl SimulatedBackend {
    /// Create a new simulated backend with the given configuration.
    ///
    /// The provided `engine` is the mix engine to drive.
    #[must_use]
    pub fn new(engine: MixEngine, sample_rate: u32, buffer_frames: u32) -> Self {
        // Safety: unwrap_or_else is not clippy::unwrap_used
        let inner = SimState {
            engine,
            output_buffer: core::array::from_fn(|_| [Vec::new()]),
            xrun_count: 0,
            frames_processed: 0,
        };
        Self {
            sample_rate,
            buffer_frames,
            active: false,
            phase: [0.0_f32; MAX_CHANNELS],
            state: Arc::new(Mutex::new(inner)),
        }
    }

    /// Process `n_frames` synchronously, accumulating output into the internal buffer.
    ///
    /// Returns [`ProcessStats`] for this run.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend is not active or the internal lock is poisoned.
    pub fn process_n_frames(&mut self, n_frames: u32) -> BackendResult<ProcessStats> {
        if !self.active {
            return Err(AudioEngineError::InvalidState {
                reason: "backend not active; call activate() first".into(),
            });
        }

        let start = Instant::now();
        let freq = 440.0_f32;
        let omega = 2.0 * core::f32::consts::PI * freq / self.sample_rate as f32;

        let mut guard = self
            .state
            .lock()
            .map_err(|_| AudioEngineError::InvalidState {
                reason: "state mutex poisoned".into(),
            })?;

        for _ in 0..n_frames {
            // Generate synthetic sample for each channel (440 Hz sine, 0.5 amplitude).
            let mut samples = [0.0_f32; MAX_CHANNELS];
            for (ch, samp) in samples.iter_mut().enumerate() {
                *samp = 0.5 * self.phase[ch].sin();
                self.phase[ch] = (self.phase[ch] + omega) % (2.0 * core::f32::consts::PI);
            }

            let frame_out = guard.engine.process_frame(&samples);

            // Store output (only keeps last `buffer_frames` per mix to bound memory).
            for (mix_idx, pair) in frame_out.mixes.iter().enumerate() {
                if let Some(slot) = guard.output_buffer.get_mut(mix_idx) {
                    slot[0].push(*pair);
                    // Bound: keep only last 4096 frames to avoid unbounded growth in tests.
                    if slot[0].len() > 4096 {
                        slot[0].drain(..1);
                    }
                }
            }

            guard.frames_processed = guard.frames_processed.saturating_add(1);
        }

        let cpu_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

        Ok(ProcessStats {
            frames: n_frames,
            xrun: false, // simulated backend never xruns
            cpu_us: Some(cpu_us),
        })
    }

    /// Returns a copy of the output sample pairs for the given mix index.
    ///
    /// Empty if the mix slot is inactive or no frames processed yet.
    ///
    /// # Errors
    ///
    /// Returns an error if the mutex is poisoned.
    pub fn output_samples(&self, mix_idx: usize) -> BackendResult<Vec<(f32, f32)>> {
        let guard = self
            .state
            .lock()
            .map_err(|_| AudioEngineError::InvalidState {
                reason: "state mutex poisoned".into(),
            })?;
        Ok(guard
            .output_buffer
            .get(mix_idx)
            .map(|slot| slot[0].clone())
            .unwrap_or_default())
    }

    /// Returns the total frames processed since activation.
    ///
    /// # Errors
    ///
    /// Returns an error if the mutex is poisoned.
    pub fn frames_processed(&self) -> BackendResult<u64> {
        let guard = self
            .state
            .lock()
            .map_err(|_| AudioEngineError::InvalidState {
                reason: "state mutex poisoned".into(),
            })?;
        Ok(guard.frames_processed)
    }

    /// Returns the current [`MixEngine`] revision.
    ///
    /// # Errors
    ///
    /// Returns an error if the mutex is poisoned.
    pub fn engine_revision(&self) -> BackendResult<u64> {
        let guard = self
            .state
            .lock()
            .map_err(|_| AudioEngineError::InvalidState {
                reason: "state mutex poisoned".into(),
            })?;
        Ok(guard.engine.revision())
    }

    /// Apply a function to the inner `MixEngine` for configuration.
    ///
    /// Must be called before `activate()`. Returns an error if active or
    /// if the mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns [`AudioEngineError::InvalidState`] if backend is already active.
    pub fn configure_engine<F>(&self, f: F) -> BackendResult<()>
    where
        F: FnOnce(&mut MixEngine) -> Result<(), mix_engine::mix_engine::EngineError>,
    {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| AudioEngineError::InvalidState {
                reason: "state mutex poisoned".into(),
            })?;
        f(&mut guard.engine).map_err(AudioEngineError::from)
    }
}

impl Backend for SimulatedBackend {
    fn activate(&mut self) -> BackendResult<()> {
        if self.active {
            return Err(AudioEngineError::InvalidState {
                reason: "already active".into(),
            });
        }
        // Reset phase accumulators on fresh activation.
        self.phase = [0.0_f32; MAX_CHANNELS];
        self.active = true;
        log::info!(
            "[SIMULATED] audio-engine activated: {}Hz, {}frames/period ({:.2}ms)",
            self.sample_rate,
            self.buffer_frames,
            f64::from(self.buffer_frames) / f64::from(self.sample_rate) * 1000.0,
        );
        Ok(())
    }

    fn deactivate(&mut self) {
        self.active = false;
        log::info!("[SIMULATED] audio-engine deactivated");
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn buffer_frames(&self) -> u32 {
        self.buffer_frames
    }

    fn name(&self) -> &'static str {
        "SimulatedBackend [SIMULATED]"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use mix_engine::{mix_send::MixSend, Channel, Mix, MixEngine};

    fn make_backend_with_active_send() -> SimulatedBackend {
        let mut engine = MixEngine::new();

        let ch = Channel::new(0, "CH01");
        engine.set_channel(0, ch).expect("set_channel");

        let mut mix = Mix::new(1, "Monitor A");
        let send = MixSend::new(0, 1);
        mix.set_send(0, send).expect("set_send");
        engine.set_mix(0, mix).expect("set_mix");

        SimulatedBackend::new(engine, SAMPLE_RATE, 256)
    }

    #[test]
    fn test_simulated_backend_name() {
        let b = SimulatedBackend::new(MixEngine::new(), SAMPLE_RATE, 256);
        assert!(b.name().contains("SIMULATED"));
    }

    #[test]
    fn test_activate_deactivate() {
        let mut b = SimulatedBackend::new(MixEngine::new(), SAMPLE_RATE, 256);
        assert!(!b.is_active());
        b.activate().expect("activate");
        assert!(b.is_active());
        b.deactivate();
        assert!(!b.is_active());
    }

    #[test]
    fn test_double_activate_error() {
        let mut b = SimulatedBackend::new(MixEngine::new(), SAMPLE_RATE, 256);
        b.activate().expect("first activate");
        let err = b.activate();
        assert!(err.is_err());
    }

    #[test]
    fn test_process_requires_active() {
        let mut b = SimulatedBackend::new(MixEngine::new(), SAMPLE_RATE, 256);
        let err = b.process_n_frames(256);
        assert!(err.is_err());
    }

    #[test]
    fn test_process_n_frames_returns_stats() {
        let mut b = make_backend_with_active_send();
        b.activate().expect("activate");
        let stats = b.process_n_frames(256).expect("process");
        assert_eq!(stats.frames, 256);
        assert!(!stats.xrun);
        assert!(stats.cpu_us.is_some());
    }

    #[test]
    fn test_frames_processed_accumulates() {
        let mut b = make_backend_with_active_send();
        b.activate().expect("activate");
        b.process_n_frames(256).expect("1st");
        b.process_n_frames(256).expect("2nd");
        let total = b.frames_processed().expect("frames_processed");
        assert_eq!(total, 512);
    }

    #[test]
    fn test_output_samples_non_empty_after_processing() {
        let mut b = make_backend_with_active_send();
        b.activate().expect("activate");
        b.process_n_frames(128).expect("process");
        let out = b.output_samples(0).expect("output_samples");
        assert_eq!(out.len(), 128);
    }

    #[test]
    fn test_output_samples_non_zero_for_active_channel() {
        let mut b = make_backend_with_active_send();
        b.activate().expect("activate");
        b.process_n_frames(64).expect("process");
        let out = b.output_samples(0).expect("out");
        // At least some samples should be non-zero (440 Hz sine into unmuted channel)
        let any_nonzero = out.iter().any(|(l, r)| l.abs() > 1e-6 || r.abs() > 1e-6);
        assert!(any_nonzero, "expected non-zero output from active channel");
    }

    #[test]
    fn test_output_samples_empty_mix_slot_is_zero() {
        let mut b = make_backend_with_active_send();
        b.activate().expect("activate");
        b.process_n_frames(64).expect("process");
        // Mix slot 1 has no mix assigned → all zeroes
        let out = b.output_samples(1).expect("out");
        let all_zero = out
            .iter()
            .all(|(l, r)| l.abs() < f32::EPSILON && r.abs() < f32::EPSILON);
        assert!(all_zero, "unassigned mix slot 1 should be silent");
    }

    #[test]
    fn test_sample_rate_and_buffer_frames() {
        let b = SimulatedBackend::new(MixEngine::new(), 48_000, 512);
        assert_eq!(b.sample_rate(), 48_000);
        assert_eq!(b.buffer_frames(), 512);
    }

    #[test]
    fn test_configure_engine_sets_revision() {
        let b = SimulatedBackend::new(MixEngine::new(), SAMPLE_RATE, 256);
        let rev_before = b.engine_revision().expect("rev");
        assert_eq!(rev_before, 0);
        b.configure_engine(|engine| {
            let ch = Channel::new(0, "GUITAR");
            engine.set_channel(0, ch)
        })
        .expect("configure");
        let rev_after = b.engine_revision().expect("rev");
        assert_eq!(rev_after, 1);
    }
}
