//! Combined fault profile — chains multiple profiles in sequence.
//!
//! Each stage is applied to the output of the previous stage.  This lets
//! callers compose realistic network degradations (loss → reorder → duplicate)
//! without incurring dynamic dispatch overhead.

use crate::{
    DuplicateProfile, FaultError, JitterProfile, LossProfile, OutageProfile, Packet, ReorderProfile,
};

/// A single stage in a [`CombinedFaultProfile`] pipeline.
///
/// Each variant wraps one of the concrete profile types so the pipeline
/// remains enum-dispatched (no `dyn` trait objects) and fully deterministic.
#[derive(Debug, Clone)]
pub enum Stage {
    /// Apply fixed-rate packet loss.
    Loss(LossProfile),
    /// Apply fixed-interval packet reordering.
    Reorder(ReorderProfile),
    /// Apply duplicate packet injection.
    Duplicate(DuplicateProfile),
    /// Apply fixed-delay jitter.
    Jitter(JitterProfile),
    /// Apply a contiguous-window outage.
    Outage(OutageProfile),
}

impl Stage {
    /// Apply this stage to `packets` and return the resulting packet sequence.
    #[must_use]
    fn apply(&self, packets: &[Packet]) -> Vec<Packet> {
        match self {
            Stage::Loss(p) => p.apply(packets).delivered,
            Stage::Reorder(p) => p.apply(packets).delivered,
            Stage::Duplicate(p) => p.apply(packets),
            Stage::Jitter(p) => p.apply(packets).delivered,
            Stage::Outage(p) => p.apply(packets).delivered,
        }
    }
}

/// A pipeline of fault profiles applied in order.
///
/// Stages are applied left-to-right: the output of each stage becomes the
/// input of the next.  At least one stage is required; construction fails
/// on an empty stage list.
///
/// # Example
///
/// ```rust
/// use network_fault::{CombinedFaultProfile, Stage, LossProfile, DuplicateProfile, Packet};
///
/// let combined = CombinedFaultProfile::new(vec![
///     Stage::Loss(LossProfile::new(4).unwrap()),
///     Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
/// ]).unwrap();
///
/// let packets = Packet::burst(1, 12);
/// let result = combined.apply(&packets);
/// assert!(!result.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct CombinedFaultProfile {
    stages: Vec<Stage>,
}

impl CombinedFaultProfile {
    /// Construct a combined profile from an ordered list of stages.
    ///
    /// # Errors
    /// [`FaultError::InvalidParameter`] when `stages` is empty.
    pub fn new(stages: Vec<Stage>) -> Result<Self, FaultError> {
        if stages.is_empty() {
            return Err(FaultError::InvalidParameter(
                "CombinedFaultProfile requires at least one stage",
            ));
        }
        Ok(Self { stages })
    }

    /// Apply all stages in order to `packets`, returning the final sequence.
    ///
    /// Each stage receives the output of the previous stage as its input.
    #[must_use]
    pub fn apply(&self, packets: &[Packet]) -> Vec<Packet> {
        let mut current = packets.to_vec();
        for stage in &self.stages {
            current = stage.apply(&current);
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn burst(n: usize) -> Vec<Packet> {
        Packet::burst(1, n)
    }

    #[test]
    fn empty_stages_rejected() {
        assert!(CombinedFaultProfile::new(vec![]).is_err());
    }

    #[test]
    fn single_stage_loss_works() {
        // 8 packets, LossProfile(4) drops positions 4 and 8 → 6 delivered.
        let profile =
            CombinedFaultProfile::new(vec![Stage::Loss(LossProfile::new(4).unwrap())]).unwrap();
        let result = profile.apply(&burst(8));
        assert_eq!(result.len(), 6);
        // Sequences 4 and 8 dropped.
        let seqs: Vec<u64> = result.iter().map(|p| p.sequence).collect();
        assert_eq!(seqs, vec![1, 2, 3, 5, 6, 7]);
    }

    #[test]
    fn chain_loss_then_reorder() {
        // 12 packets → loss(4) drops 3 → 9 remain → reorder(3) swaps pairs.
        let profile = CombinedFaultProfile::new(vec![
            Stage::Loss(LossProfile::new(4).unwrap()),
            Stage::Reorder(ReorderProfile::new(3).unwrap()),
        ])
        .unwrap();
        let result = profile.apply(&burst(12));
        // Loss drops positions 4,8,12 → 9 delivered.
        assert_eq!(result.len(), 9, "9 packets survive loss stage");
        // Reorder should not drop any.
        let mut seqs: Vec<u64> = result.iter().map(|p| p.sequence).collect();
        seqs.sort_unstable();
        // No duplicates, no missing beyond what loss removed.
        let unique_count = {
            seqs.dedup();
            seqs.len()
        };
        assert_eq!(unique_count, 9, "no packets lost or duplicated by reorder");
    }

    #[test]
    fn chain_loss_then_duplicate() {
        // 12 packets → loss(4) drops 3 → 9 remain → duplicate(3) adds 3 → 12.
        let profile = CombinedFaultProfile::new(vec![
            Stage::Loss(LossProfile::new(4).unwrap()),
            Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
        ])
        .unwrap();
        let result = profile.apply(&burst(12));
        // 9 after loss, then every 3rd duplicated (positions 3,6,9) → 3 extra.
        assert_eq!(result.len(), 12, "count increases after duplication");
    }
}
