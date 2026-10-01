use bitview_cohort::{CohortContext, CreationCohorts};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlockCumulativeWithSums, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::CumulativeCreationSources;

#[derive(Traversable)]
pub struct CumulativeRealizedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// Includes spends grouped by the address's pre-spend balance.
    pub cohorts: CreationCohorts<LazyFiatPerBlockCumulativeWithSums<Cents>>,
    #[traversable(hidden)]
    pub stored: CumulativeCreationSources<Cents, M>,
}

impl CumulativeRealizedByCohort {
    pub fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = CumulativeCreationSources::forced_import(
            db,
            &format!("{metric}_cumulative_cents"),
            version,
        )?;
        let cohorts = CreationCohorts::new(|cohort_id| {
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
