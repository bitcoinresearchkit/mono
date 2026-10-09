use brk_types::{Height, OutputType};

use crate::addr::{AddrTypeToAddrCount, AddrTypeToSupply, add_type_delta};

use super::ExposedAddrVecs;

/// Runtime running totals for exposed-address tracking.
#[derive(Debug, Default, Clone)]
pub struct ExposedAddrState {
    pub funded: AddrTypeToAddrCount,
    pub total: AddrTypeToAddrCount,
    pub supply: AddrTypeToSupply,
}

impl ExposedAddrState {
    /// Adds the change to `output_type`'s values of a shard that started from `base`.
    pub fn add_type_delta(&mut self, shard: &Self, base: &Self, output_type: OutputType) {
        let Self {
            funded,
            total,
            supply,
        } = shard;
        add_type_delta(&mut self.funded, funded, &base.funded, output_type);
        add_type_delta(&mut self.total, total, &base.total, output_type);
        add_type_delta(&mut self.supply, supply, &base.supply, output_type);
    }
}

impl From<(&ExposedAddrVecs, Height)> for ExposedAddrState {
    #[inline]
    fn from((vecs, starting_height): (&ExposedAddrVecs, Height)) -> Self {
        Self {
            funded: AddrTypeToAddrCount::from((&vecs.count.funded, starting_height)),
            total: AddrTypeToAddrCount::from((&vecs.count.total, starting_height)),
            supply: AddrTypeToSupply::from((&vecs.supply, starting_height)),
        }
    }
}
