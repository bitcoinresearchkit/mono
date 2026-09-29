use crate::metrics::CohortMetrics;
use bitview_cohort::AgeRange;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_types::StoredF64;

/// The replay loop can write origin metrics but cannot access address state.
pub struct OriginTargets<'a> {
    pub cohorts: &'a mut CohortMetrics,
    pub coindays_created: &'a mut AgeRange<PerBlockCumulativeRolling<StoredF64>>,
    pub coinblocks_destroyed: &'a mut PerBlockCumulativeRolling<StoredF64>,
}
