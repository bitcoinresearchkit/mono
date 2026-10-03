use std::ops::AddAssign;

use bitview_cohort::{CohortContext, CohortGroup};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::StoredU64;
use bitview_traversable::Traversable;
use bitview_vecs::{CumulativeCohortSources, LazyPerBlockCumulativeRolling, LazyWindowStartVec};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct SpentOutputCount<G: CohortGroup, M: StorageMode = Rw> {
    #[traversable(flatten)]
    cohorts: G::Of<LazyPerBlockCumulativeRolling<StoredU64>>,
    #[traversable(hidden)]
    pub stored: CumulativeCohortSources<G, StoredU64, M>,
}

impl<G: CohortGroup> SpentOutputCount<G>
where
    G::Of<StoredU64>: AddAssign + Clone + Default,
{
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let stored = CumulativeCohortSources::import(db, "spent_utxo_count_cumulative", version)?;
        let cohorts = G::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "spent_utxo_count");
            LazyPerBlockCumulativeRolling::from_cumulative_source(
                &name,
                version,
                stored
                    .stored
                    .get(cohort_id)
                    .expect("spent-output cohort source"),
                window_starts,
                mappings,
            )
        });
        Ok(Self { cohorts, stored })
    }
}
