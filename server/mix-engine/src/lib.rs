//! Open IEM Platform — Mix Engine
//!
//! Realtime-safe mix engine core. All types in this crate are pure data and
//! pure DSP computation: **no I/O, no heap allocation in the audio path,
//! no mutex with unbounded hold time**.
//!
//! # Architecture
//!
//! ```text
//! Channel (input source)
//!     └─► MixSend (per-mix routing config: gain, pan, mute, solo)
//!             └─► Mix (summed stereo output with master gain + limiter)
//!                     └─► MixEngine (collection of all channels + mixes)
//! ```
//!
//! # Realtime Safety Rules (ENFORCED)
//!
//! The following are FORBIDDEN inside `process()` and any function called from
//! the audio callback:
//! - File or network I/O
//! - Heap allocation (`Box::new`, `Vec::push` with reallocation, `String::new`, etc.)
//! - `Mutex::lock` without a bounded timeout (prefer lock-free structures)
//! - `println!` / `eprintln!` (syscalls)
//! - `thread::sleep` or any blocking wait
//! - Panic paths that allocate (format strings with `{}` on heap types)
//!
//! Control-plane updates (fader moves, mute toggles) must be communicated to
//! the audio thread via a lock-free channel (e.g., `crossbeam-channel`
//! bounded SPSC). That infrastructure lives in a future `mix-engine-rt` crate.
//! This crate contains the pure state and computation only.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]
#![allow(clippy::indexing_slicing)]

pub mod channel;
pub mod compressor;
pub mod eq;
pub mod limiter;
pub mod mix;
pub mod mix_engine;
pub mod mix_send;

pub use channel::Channel;
pub use compressor::Compressor;
pub use eq::{EqBand, ParametricEq, MAX_EQ_BANDS};
pub use limiter::Limiter;
pub use mix::Mix;
pub use mix_engine::{EngineError, FrameOutput, MixEngine};
pub use mix_send::MixSend;

// ---------------------------------------------------------------------------
// Shared audio format constants
// ---------------------------------------------------------------------------

/// Target sample rate (Hz).
pub const SAMPLE_RATE: u32 = 48_000;

/// Maximum channels supported by the engine (MVP = 8).
pub const MAX_CHANNELS: usize = 8;

/// Maximum independent mixes supported by the engine (MVP = 2).
pub const MAX_MIXES: usize = 2;

/// Minimum gain in dB (represents –∞, implemented as silence).
pub const GAIN_DB_MIN: f32 = -144.0;

/// Maximum gain in dB.
pub const GAIN_DB_MAX: f32 = 12.0;

/// 0 dB as a linear amplitude multiplier.
pub const GAIN_UNITY: f32 = 1.0;

/// Pan hard-left.
pub const PAN_LEFT: f32 = -1.0;

/// Pan centre.
pub const PAN_CENTER: f32 = 0.0;

/// Pan hard-right.
pub const PAN_RIGHT: f32 = 1.0;

/// Convert a dB value to a linear amplitude multiplier.
///
/// Values ≤ [`GAIN_DB_MIN`] return exactly `0.0` (silence / –∞ dB).
///
/// # Examples
///
/// ```
/// use mix_engine::{db_to_linear, GAIN_DB_MIN};
/// assert!((mix_engine::db_to_linear(0.0) - 1.0).abs() < 1e-6);
/// assert_eq!(mix_engine::db_to_linear(GAIN_DB_MIN), 0.0);
/// ```
#[must_use]
#[inline]
pub fn db_to_linear(db: f32) -> f32 {
    if db <= GAIN_DB_MIN {
        0.0
    } else {
        10.0_f32.powf(db / 20.0)
    }
}

/// Convert a linear amplitude multiplier to dB.
///
/// Values ≤ `0.0` return [`GAIN_DB_MIN`].
///
/// # Examples
///
/// ```
/// use mix_engine::{linear_to_db, GAIN_DB_MIN};
/// assert!((mix_engine::linear_to_db(1.0) - 0.0).abs() < 1e-5);
/// assert_eq!(mix_engine::linear_to_db(0.0), GAIN_DB_MIN);
/// ```
#[must_use]
#[inline]
pub fn linear_to_db(linear: f32) -> f32 {
    if linear <= 0.0 {
        GAIN_DB_MIN
    } else {
        20.0 * linear.log10()
    }
}

/// Apply equal-power stereo pan to a mono sample.
///
/// Returns `(left, right)` using the sine/cosine law:
/// - pan = -1.0 → full left
/// - pan =  0.0 → centre (-3 dB each)
/// - pan = +1.0 → full right
///
/// Pan value is clamped to `[-1.0, 1.0]`.
///
/// # Examples
///
/// ```
/// use mix_engine::apply_pan;
/// let (l, r) = apply_pan(1.0, 0.0);
/// assert!((l - r).abs() < 1e-5); // centre: equal power
/// let (l, r) = apply_pan(1.0, -1.0);
/// assert!(l > 0.9); // hard left
/// assert!(r < 1e-5);
/// ```
#[must_use]
#[inline]
pub fn apply_pan(sample: f32, pan: f32) -> (f32, f32) {
    let pan_c = pan.clamp(PAN_LEFT, PAN_RIGHT);
    // Map [-1, 1] → [0, π/2] for sine/cosine law
    let angle = (pan_c + 1.0) * (core::f32::consts::FRAC_PI_4);
    let left = angle.cos() * sample;
    let right = angle.sin() * sample;
    (left, right)
}
