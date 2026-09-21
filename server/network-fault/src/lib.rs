//! Deterministic network fault profiles for the Open IEM Platform.
//!
//! All profiles are **L1 SIMULATED** — they operate on in-memory packet
//! sequences and never touch real network I/O.  They exist solely to
//! exercise the receiver, recovery and observability crates under controlled
//! degradation, producing verifiable thresholds without hardware.
//!
//! # Design
//!
//! - **No I/O, no async, no heap beyond `Vec`/`VecDeque`** in the pipeline.
//! - Bounded inputs: all profile parameters are validated at construction.
//! - Deterministic: same seed → same outcome every run.
//!
//! # Profiles
//!
//! | Profile | What it injects |
//! |---------|-----------------|
//! | [`loss`] | Fixed-rate packet loss (drop every Nth packet). |
//! | [`jitter`] | Fixed delay added to every Nth packet (sequence reordering). |
//! | [`reorder`] | Swap adjacent pair every Nth position. |
//! | [`outage`] | Drop a contiguous window of packets (link outage). |
//! | [`reconnect`] | Split stream at a point, pass state through [`RecoveryRegistry`], resume. |
//! | [`duplicate`] | Replay selected packets (duplicate sequence injection). |
//! | [`combined`] | Chain multiple profiles in sequence. |

#![deny(missing_docs, unsafe_code)]

pub mod combined;
pub mod duplicate;
pub mod jitter;
pub mod loss;
pub mod outage;
pub mod reconnect;
pub mod reorder;

pub use combined::{CombinedFaultProfile, Stage};
pub use duplicate::DuplicateProfile;
pub use jitter::JitterProfile;
pub use loss::LossProfile;
pub use outage::OutageProfile;
pub use reconnect::ReconnectProfile;
pub use reorder::ReorderProfile;

use thiserror::Error;

/// Errors returned when constructing or running a fault profile.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FaultError {
    /// A profile parameter was zero or exceeded the allowed maximum.
    #[error("invalid profile parameter: {0}")]
    InvalidParameter(&'static str),
}

/// A synthetic packet: sequence number and a fixed-size payload stub.
///
/// Real codec data is replaced by a 1-byte sentinel so tests remain
/// allocation-minimal and independent of Opus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    /// Monotonically increasing sequence counter assigned by the sender.
    pub sequence: u64,
    /// Payload bytes (at least 1, at most 1500).
    pub payload: Vec<u8>,
}

impl Packet {
    /// Create a packet with a 1-byte sentinel payload.
    #[must_use]
    pub fn sentinel(sequence: u64) -> Self {
        Self {
            sequence,
            payload: vec![0xAB],
        }
    }

    /// Create a burst of `count` sequential packets starting at `start_seq`.
    ///
    /// # Panics
    /// Never — `count` is bounded by callers.
    #[must_use]
    pub fn burst(start_seq: u64, count: usize) -> Vec<Self> {
        (0..count)
            .map(|i| Self::sentinel(start_seq + i as u64))
            .collect()
    }
}

/// Run-result produced by a profile simulation.
#[derive(Debug, Clone)]
pub struct SimResult {
    /// Packets delivered to the receiver (in arrival order).
    pub delivered: Vec<Packet>,
    /// Packets dropped (lost or suppressed by outage).
    pub dropped: usize,
    /// Packets that arrived out of original sequence order.
    pub reordered: usize,
    /// Packets that experienced injected delay (jitter events).
    pub jitter_events: usize,
}
