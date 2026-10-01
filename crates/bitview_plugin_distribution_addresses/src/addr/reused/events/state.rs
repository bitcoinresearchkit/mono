use bitview_cohort::ByAddrType;
use derive_more::{Deref, DerefMut};

/// Per-block running counter of address-reuse events, per address type. Shared
/// across reused (receive-based) and respent (spend-based) flavors, and
/// across output-side ("output landed on a previously-used address") and
/// input-side ("input spent from an address in the set") event kinds.
///
/// Reset at the start of each block; no disk recovery needed since per-block
/// flow is reconstructed deterministically from `process_received` /
/// `process_sent`.
#[derive(Debug, Default, Deref, DerefMut)]
pub struct AddrTypeToAddrEventCount(ByAddrType<u64>);

impl AddrTypeToAddrEventCount {
    #[inline]
    pub fn sum(&self) -> u64 {
        self.0.values().sum()
    }

    #[inline]
    pub fn reset(&mut self) {
        for v in self.0.values_mut() {
            *v = 0;
        }
    }
}
