use brk_types::Height;

use crate::addr::{AddrTypeToAddrCount, AddrTypeToSupply};

use super::{AddrTypeToAddrEventCount, ReusedAddrVecs};

/// Runtime running totals for receive-based reuse or spend-based respending.
#[derive(Debug, Default)]
pub struct ReusedAddrState {
    pub funded: AddrTypeToAddrCount,
    pub total: AddrTypeToAddrCount,
    pub supply: AddrTypeToSupply,
    pub output_events: AddrTypeToAddrEventCount,
    pub input_events: AddrTypeToAddrEventCount,
    pub active: AddrTypeToAddrEventCount,
}

impl ReusedAddrState {
    #[inline]
    pub fn reset_per_block(&mut self) {
        self.output_events.reset();
        self.input_events.reset();
        self.active.reset();
    }
}

impl From<(&ReusedAddrVecs, Height)> for ReusedAddrState {
    #[inline]
    fn from((vecs, starting_height): (&ReusedAddrVecs, Height)) -> Self {
        Self {
            funded: AddrTypeToAddrCount::from((&vecs.count.funded, starting_height)),
            total: AddrTypeToAddrCount::from((&vecs.count.total, starting_height)),
            supply: AddrTypeToSupply::from((&vecs.supply, starting_height)),
            output_events: AddrTypeToAddrEventCount::default(),
            input_events: AddrTypeToAddrEventCount::default(),
            active: AddrTypeToAddrEventCount::default(),
        }
    }
}
