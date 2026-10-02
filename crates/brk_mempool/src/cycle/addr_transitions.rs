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

#[cfg(test)]
mod tests {
    use bitcoin::{ScriptBuf, WPubkeyHash, hashes::Hash};

    use super::*;

    fn addr(seed: u8) -> AddrBytes {
        let mut bytes = [0u8; 20];
        bytes[0] = seed;
        let script = ScriptBuf::new_p2wpkh(&WPubkeyHash::from_byte_array(bytes));
        AddrBytes::try_from(&script).expect("p2wpkh -> AddrBytes")
    }

    #[test]
    fn same_cycle_cancellation_keeps_distinct_addresses_independent() {
        let mut transitions = AddrTransitions::default();
        for (seed, changes) in [
            (3, &[true, false, true][..]),
            (4, &[false, true, false][..]),
            (5, &[true][..]),
            (6, &[false][..]),
        ] {
            for &enters in changes {
                if enters {
                    transitions.record_enter(addr(seed));
                } else {
                    transitions.record_leave(addr(seed));
                }
            }
        }
        let (mut enters, mut leaves) = transitions.into_vecs();
        enters.sort_by_key(|x| x.as_slice()[0]);
        leaves.sort_by_key(|x| x.as_slice()[0]);
        assert_eq!(enters, [addr(3), addr(5)]);
        assert_eq!(leaves, [addr(4), addr(6)]);
    }
}
