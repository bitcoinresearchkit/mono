use brk_types::Height;

use crate::addr::{AddrTypeToAddrCount, AddrTypeToSupply};

use super::ExposedAddrVecs;

/// Runtime running totals for exposed-address tracking.
#[derive(Debug, Default)]
pub struct ExposedAddrState {
    pub funded: AddrTypeToAddrCount,
    pub total: AddrTypeToAddrCount,
    pub supply: AddrTypeToSupply,
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
