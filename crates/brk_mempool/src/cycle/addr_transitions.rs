//! Per-cycle 0↔1+ address transition buffer. Same-cycle cancellation
//! (enter→leave, leave→enter) is encapsulated on the recording methods.

use brk_types::AddressBytes;
use rustc_hash::FxHashSet;

#[derive(Default)]
pub struct AddrTransitions {
    enters: FxHashSet<AddressBytes>,
    leaves: FxHashSet<AddressBytes>,
}

impl AddrTransitions {
    pub fn record_enter(&mut self, bytes: AddressBytes) {
        if !self.leaves.remove(&bytes) {
            self.enters.insert(bytes);
        }
    }

    pub fn record_leave(&mut self, bytes: AddressBytes) {
        if !self.enters.remove(&bytes) {
            self.leaves.insert(bytes);
        }
    }

    pub fn into_vecs(self) -> (Vec<AddressBytes>, Vec<AddressBytes>) {
        (
            self.enters.into_iter().collect(),
            self.leaves.into_iter().collect(),
        )
    }
}
