//! Audio backend abstraction.
//!
//! A `Backend` owns the audio I/O lifecycle: it activates, drives the process
//! callback with real (JACK) or synthetic (Simulated) audio, and deactivates
//! cleanly. The DSP computation is always delegated to [`mix_engine::MixEngine`].
//!
//! # Available backends
//!
//! | Backend    | Feature gate | Environment          |
//! |------------|--------------|----------------------|
//! | Simulated  | (always)     | VPS / CI             |
//! | Jack       | `jack`       | RPi 5 + pipewire-jack |

pub mod simulated;

#[cfg(feature = "alsa")]
pub mod alsa_backend;

#[cfg(feature = "jack")]
pub mod jack;

use crate::error::AudioEngineError;

/// Result type for backend operations.
pub type BackendResult<T> = Result<T, AudioEngineError>;

/// Statistics emitted by the backend after each processing cycle.
///
/// Used for monitoring and XRUN detection (see `docs/audio/AUDIO-SLA.md`).
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessStats {
    /// Number of frames processed in this cycle.
    pub frames: u32,
    /// Whether an XRUN was detected in this cycle.
    pub xrun: bool,
    /// CPU time consumed in this cycle (µs). `None` if not measured.
    pub cpu_us: Option<u64>,
}

/// Trait for audio backend implementations.
///
/// # Thread safety
///
/// Implementors must be `Send` — the backend is moved to the processing thread.
/// The audio callback itself runs in a separate OS thread managed by the backend.
pub trait Backend: Send {
    /// Start the audio backend and begin processing.
    ///
    /// # Errors
    ///
    /// Returns [`AudioEngineError::BackendActivate`] if the backend cannot start.
    fn activate(&mut self) -> BackendResult<()>;

    /// Stop the audio backend and release resources.
    fn deactivate(&mut self);

    /// Returns `true` if the backend is currently running.
    fn is_active(&self) -> bool;

    /// Returns the actual sample rate negotiated with the audio system.
    /// May differ from the requested rate after negotiation.
    fn sample_rate(&self) -> u32;

    /// Returns the actual buffer size (frames per period) in use.
    fn buffer_frames(&self) -> u32;

    /// Human-readable name for this backend (for logging).
    fn name(&self) -> &'static str;
}
