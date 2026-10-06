use brk_types::{Height, OutputType};

use crate::addr::{AddrTypeToAddrCount, AddrTypeToSupply};

use super::{AddrTypeToAddrEventCount, ReusedAddrVecs};

/// Runtime running totals for receive-based reuse or spend-based respending.
#[derive(Debug, Default, Clone)]
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

    /// Takes `output_type`'s values from `other`.
    pub fn copy_type(&mut self, other: &Self, output_type: OutputType) {
        let Self {
            funded,
            total,
            supply,
            output_events,
            input_events,
            active,
        } = other;
        *self.funded.get_mut_unwrap(output_type) = *funded.get_unwrap(output_type);
        *self.total.get_mut_unwrap(output_type) = *total.get_unwrap(output_type);
        *self.supply.get_mut_unwrap(output_type) = *supply.get_unwrap(output_type);
        *self.output_events.get_mut_unwrap(output_type) = *output_events.get_unwrap(output_type);
        *self.input_events.get_mut_unwrap(output_type) = *input_events.get_unwrap(output_type);
        *self.active.get_mut_unwrap(output_type) = *active.get_unwrap(output_type);
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
