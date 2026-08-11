use brk_types::{Cents, Sats};

#[derive(Clone, Copy)]
pub(crate) struct WeightedCohortContribution {
    pub weighted_supply: Sats,
    pub complement_supply: Sats,
    pub weighted_cap: Cents,
}
