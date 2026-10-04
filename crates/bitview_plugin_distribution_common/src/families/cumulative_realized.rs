use std::ops::AddAssign;

use bitview_cohort::{CohortContext, CohortGroup};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CumulativeCohortSources, LazyFiatPerBlockCumulativeWithSums, LazyWindowStartVec,
};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct CumulativeRealizedByCohort<G: CohortGroup, M: StorageMode = Rw> {
    #[traversable(flatten)]
    cohorts: G::Of<LazyFiatPerBlockCumulativeWithSums<Cents>>,
    #[traversable(hidden)]
    pub stored: CumulativeCohortSources<G, Cents, M>,
}

impl<G: CohortGroup> CumulativeRealizedByCohort<G>
where
    G::Of<Cents>: AddAssign + Clone + Default,
{
    pub fn import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored =
            CumulativeCohortSources::import(db, &format!("{metric}_cumulative_cents"), version)?;
        let cohorts = G::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let source = stored
                .stored
                .get(cohort_id)
                .expect("supported stored realized cohort");
            LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &name,
                version,
                source,
                mappings,
                window_starts,
            )
        });
        Ok(Self { cohorts, stored })
    }
}
