use bitview_cohort::{CohortContext, CreationCohorts};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockCumulativeRolling, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{StoredU64, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::CumulativeCreationSources;

#[derive(Traversable)]
pub struct SpentOutputCount<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: CreationCohorts<LazyPerBlockCumulativeRolling<StoredU64>>,
    #[traversable(hidden)]
    pub stored: CumulativeCreationSources<StoredU64, M>,
}

impl SpentOutputCount {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let stored =
            CumulativeCreationSources::forced_import(db, "spent_utxo_count_cumulative", version)?;
        let cohorts = CreationCohorts::new(|cohort_id| {
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
