use crate::cycle::{AddrTransitions, TxAdded, TxRemoved};

/// Per-cycle accumulator threaded through the pipeline steps and
/// drained into the public [`crate::Cycle`] at end of cycle.
#[derive(Default)]
pub struct CycleDiff {
    pub added: Vec<TxAdded>,
    pub removed: Vec<TxRemoved>,
    pub addrs: AddrTransitions,
}

impl CycleDiff {
    pub fn membership_changed(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty()
    }
}
