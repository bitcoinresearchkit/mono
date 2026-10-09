use brk_types::{Height, OutputType};

use crate::addr::{AddrTypeToAddrCount, AddrTypeToSupply, add_type_delta};

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

    /// Adds the change to `output_type`'s values of a shard that started from `base`.
    pub fn add_type_delta(&mut self, shard: &Self, base: &Self, output_type: OutputType) {
        let Self {
            funded,
            total,
            supply,
            output_events,
            input_events,
            active,
        } = shard;
        add_type_delta(&mut self.funded, funded, &base.funded, output_type);
        add_type_delta(&mut self.total, total, &base.total, output_type);
        add_type_delta(&mut self.supply, supply, &base.supply, output_type);
        add_type_delta(
            &mut self.output_events,
            output_events,
            &base.output_events,
            output_type,
        );
        add_type_delta(
            &mut self.input_events,
            input_events,
            &base.input_events,
            output_type,
        );
        add_type_delta(&mut self.active, active, &base.active, output_type);
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
