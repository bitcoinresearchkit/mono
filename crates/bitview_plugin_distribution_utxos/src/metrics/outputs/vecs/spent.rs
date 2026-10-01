use crate::cumulative::CumulativeUtxoSources;
use crate::groups::UtxoGroups;
use bitview_cohort::CohortContext;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockCumulativeRolling, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{StoredU64, Version};
use vecdb::{Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct SpentOutputCount<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UtxoGroups<LazyPerBlockCumulativeRolling<StoredU64>>,
    #[traversable(hidden)]
    pub stored: CumulativeUtxoSources<StoredU64, M>,
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
            CumulativeUtxoSources::forced_import(db, "spent_utxo_count_cumulative", version)?;
        let cohorts = UtxoGroups::new(|cohort_id| {
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
