//! Duplicate packet injection profile.
//!
//! Replays selected packets at configurable positions in the stream.
//! No I/O, no async. L1 SIMULATED.

use crate::{FaultError, Packet};

/// Injects duplicate packets at regular intervals.
///
/// Every `interval`-th packet (1-indexed) is replayed immediately after
/// its original position. The duplicated copy carries the same sequence
/// number and payload.
#[derive(Debug, Clone)]
pub struct DuplicateProfile {
    interval: usize,
}

impl DuplicateProfile {
    /// Create a profile that duplicates every `interval`-th packet.
    ///
    /// # Errors
    /// Returns [`FaultError::InvalidParameter`] if `interval` is zero.
    pub fn new(interval: usize) -> Result<Self, FaultError> {
        if interval == 0 {
            return Err(FaultError::InvalidParameter("interval must be > 0"));
        }
        Ok(Self { interval })
    }

    /// Apply duplicates to `packets`, returning the expanded sequence.
    /// Original packets are preserved in order; duplicates are inserted
    /// immediately after their original.
    #[must_use]
    pub fn apply(&self, packets: &[Packet]) -> Vec<Packet> {
        let mut out = Vec::with_capacity(packets.len() * 2);
        for (i, pkt) in packets.iter().enumerate() {
            out.push(pkt.clone());
            if (i + 1) % self.interval == 0 {
                out.push(pkt.clone()); // duplicate: same sequence, same payload
            }
        }
        out
    }
}
