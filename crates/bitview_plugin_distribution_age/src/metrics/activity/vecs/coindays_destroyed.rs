use bitview_cohort::{CohortContext, CreationCohorts};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::StoredF64;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockCumulativeRolling, LazyWindowStartVec};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::CumulativeCreationSources;

#[derive(Traversable)]
pub struct CoindaysDestroyedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: CreationCohorts<LazyPerBlockCumulativeRolling<StoredF64>>,
    #[traversable(hidden)]
    pub stored: CumulativeCreationSources<StoredF64, M>,
}

impl CoindaysDestroyedByCohort {
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored =
            CumulativeCreationSources::import(db, "coindays_destroyed_cumulative", version)?;
        let cohorts = CreationCohorts::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "coindays_destroyed");
            let source = stored
                .stored
                .get(cohort_id)
                .expect("supported coindays-destroyed cohort");
            LazyPerBlockCumulativeRolling::from_cumulative_source(
                &name,
                version,
                source,
                window_starts,
                mappings,
            )
        });
        Ok(Self { cohorts, stored })
    }
}
