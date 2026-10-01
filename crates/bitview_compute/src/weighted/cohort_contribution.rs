use brk_types::{Cents, Sats};

#[derive(Clone, Copy)]
pub struct WeightedCohortContribution {
    pub(crate) weighted_supply: Sats,
    pub(crate) complement_supply: Sats,
    pub(crate) weighted_cap: Cents,
}
