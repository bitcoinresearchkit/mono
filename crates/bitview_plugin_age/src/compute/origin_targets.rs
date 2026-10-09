use bitview_cohort::AgeRange;
use bitview_primitives::{CoinBlocks, CoinDays};
use bitview_vecs::PerBlockCumulativeRolling;

use crate::metrics::CohortMetrics;

/// The replay loop can write origin metrics but cannot access address state.
pub struct OriginTargets<'a> {
    pub cohorts: &'a mut CohortMetrics,
    pub coindays_created: &'a mut AgeRange<PerBlockCumulativeRolling<CoinDays>>,
    pub coinblocks_destroyed: &'a mut PerBlockCumulativeRolling<CoinBlocks>,
}
