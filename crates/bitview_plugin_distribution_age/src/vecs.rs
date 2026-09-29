mod compute;
mod import;

use crate::{AllChainSources, live::LiveState, metrics::CohortMetrics};
use bitview_cohort::AgeRange;
use bitview_traversable::Traversable;
use bitview_urpd::AgeBoundsMetrics;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_types::StoredF64;
use vecdb::{Database, Rw, StorageMode};

/// Age-derived metrics and resident state, independent of address state.
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState>>,
    pub cohorts: CohortMetrics<M>,
    #[traversable(wrap = "cohorts/urpd")]
    pub age_bounds: AgeBoundsMetrics<M>,
    #[traversable(wrap = "cointime/age_range")]
    pub coindays_created: AgeRange<PerBlockCumulativeRolling<StoredF64, M>>,
    #[traversable(wrap = "cointime/activity")]
    pub coinblocks_destroyed: PerBlockCumulativeRolling<StoredF64, M>,
}

impl Vecs {
    pub fn all_chain_sources(&self) -> AllChainSources {
        AllChainSources::new(self.cohorts.all_supply(), self.cohorts.all_market_cap())
    }
}
