use brk_types::{Height, OutputType};

use crate::addr::{AddrTypeToAddrCount, AddrTypeToSupply};

use super::ExposedAddrVecs;

/// Runtime running totals for exposed-address tracking.
#[derive(Debug, Default, Clone)]
pub struct ExposedAddrState {
    pub funded: AddrTypeToAddrCount,
    pub total: AddrTypeToAddrCount,
    pub supply: AddrTypeToSupply,
}

impl ExposedAddrState {
    /// Takes `output_type`'s values from `other`.
    pub fn copy_type(&mut self, other: &Self, output_type: OutputType) {
        let Self {
            funded,
            total,
            supply,
        } = other;
        *self.funded.get_mut_unwrap(output_type) = *funded.get_unwrap(output_type);
        *self.total.get_mut_unwrap(output_type) = *total.get_unwrap(output_type);
        *self.supply.get_mut_unwrap(output_type) = *supply.get_unwrap(output_type);
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
