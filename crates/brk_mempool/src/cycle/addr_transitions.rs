//! Per-cycle 0↔1+ address transition buffer. Same-cycle cancellation
//! (enter→leave, leave→enter) is encapsulated on the recording methods.

use brk_types::AddrBytes;
use rustc_hash::FxHashSet;

#[derive(Default)]
pub struct AddrTransitions {
    enters: FxHashSet<AddrBytes>,
    leaves: FxHashSet<AddrBytes>,
}

impl AddrTransitions {
    pub fn record_enter(&mut self, bytes: AddrBytes) {
        if !self.leaves.remove(&bytes) {
            self.enters.insert(bytes);
        }
    }

    pub fn record_leave(&mut self, bytes: AddrBytes) {
        if !self.enters.remove(&bytes) {
            self.leaves.insert(bytes);
        }
    }

    pub fn into_vecs(self) -> (Vec<AddrBytes>, Vec<AddrBytes>) {
        (
            self.enters.into_iter().collect(),
            self.leaves.into_iter().collect(),
        )
    }
}
