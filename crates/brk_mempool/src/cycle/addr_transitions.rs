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
    fn enter_leave_enter_collapses_to_single_enter() {
        let mut t = AddrTransitions::default();
        let a = addr(3);
        t.record_enter(a.clone());
        t.record_leave(a.clone());
        t.record_enter(a.clone());
        let (enters, leaves) = t.into_vecs();
        assert_eq!(enters, vec![a]);
        assert!(leaves.is_empty());
    }

    #[test]
    fn leave_enter_leave_collapses_to_single_leave() {
        let mut t = AddrTransitions::default();
        let a = addr(4);
        t.record_leave(a.clone());
        t.record_enter(a.clone());
        t.record_leave(a.clone());
        let (enters, leaves) = t.into_vecs();
        assert!(enters.is_empty());
        assert_eq!(leaves, vec![a]);
    }

    #[test]
    fn distinct_addrs_dont_interfere() {
        let mut t = AddrTransitions::default();
        let a = addr(5);
        let b = addr(6);
        t.record_enter(a.clone());
        t.record_leave(b.clone());
        let (mut enters, mut leaves) = t.into_vecs();
        enters.sort_by_key(|x| x.as_slice()[0]);
        leaves.sort_by_key(|x| x.as_slice()[0]);
        assert_eq!(enters, vec![a]);
        assert_eq!(leaves, vec![b]);
    }
}
