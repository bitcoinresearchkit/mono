use bitview_cohort::{CohortContext, UTXOGroupsWithoutAmountOrType};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlockCumulativeRolling, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::CumulativeUTXOCoreSources;

#[derive(Traversable)]
pub struct CumulativeValueDestroyedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<LazyFiatPerBlockCumulativeRolling<Cents>>,
    #[traversable(hidden)]
    pub stored: CumulativeUTXOCoreSources<Cents, M>,
}

impl CumulativeValueDestroyedByCohort {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let metric = "value_destroyed";
        let stored = CumulativeUTXOCoreSources::forced_import(
            db,
            "value_destroyed_cumulative_cents",
            version,
        )?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let source = stored
                .stored
                .get(cohort_id)
                .expect("supported value-destroyed cohort");
            LazyFiatPerBlockCumulativeRolling::from_cumulative_cents_source(
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
