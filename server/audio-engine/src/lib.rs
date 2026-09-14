//! Open IEM Platform — Audio Engine
//!
//! This crate bridges the [`mix_engine`] DSP core to the host audio system.
//! On hardware targets (Raspberry Pi 5) this wires into `PipeWire` via the
//! `pipewire-jack` bridge (`jack` feature). On VPS / CI it uses a simulated
//! backend that runs the DSP pipeline against a synthetic signal.
//!
//! # Architecture
//!
//! ```text
//!   [`PipeWire` graph]  ←─ JACK bridge ─→  [JackBackend]
//!         │                                     │
//!         │           (SIMULATED on VPS)        │
//!   [SimulatedBackend] ────────────────────────►│
//!                                               ↓
//!                                      [AudioEngine]
//!                                           │
//!                                           ↓
//!                                     [mix_engine::MixEngine]
//!                                     (pure DSP, no I/O)
//! ```
//!
//! # SIMULATED Label
//!
//! All items and modules tagged `SIMULATED` reflect real API surfaces with
//! correct data-flow, but do not touch real hardware. This is intentional for
//! the VPS development environment. Validation on `RPi` 5 with `PipeWire` is
//! deferred to Phase 5 (hardware integration milestone).
//!
//! # Realtime Thread Rules
//!
//! The audio callback (whether from JACK or the simulator) **must not**:
//! - Allocate on the heap
//! - Perform any I/O (file, network, `println!`)
//! - Acquire an unbounded mutex
//! - Panic with heap-formatted messages
//!
//! These rules are inherited from [`mix_engine`] and enforced here.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]

pub mod backend;
pub mod config;
pub mod engine;
pub mod error;
pub mod rt_boundary;

pub use engine::AudioEngine;
pub use error::AudioEngineError;

// Re-export constants from mix-engine for convenience.
pub use mix_engine::{MAX_CHANNELS, MAX_MIXES, SAMPLE_RATE};
