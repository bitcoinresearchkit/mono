use bitview_cohort::{CohortContext, CreationCohorts};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyValuePerBlockCumulativeRolling, LazyWindowStartVec, SatsCents};
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::metrics::CumulativeCreationValueSources;

#[derive(Traversable)]
pub struct CoreCumulativeValueByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: CreationCohorts<LazyValuePerBlockCumulativeRolling>,
    #[traversable(hidden)]
    pub stored: CumulativeCreationValueSources<M>,
}

impl CoreCumulativeValueByCohort {
    pub fn import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored =
            CumulativeCreationValueSources::import(db, &format!("{metric}_cumulative"), version)?;
        let cohorts = CreationCohorts::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let SatsCents { sats, cents } = stored
                .sources(cohort_id, &name, version)
                .expect("supported core stored value cohort");
            LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                &name,
                version,
                &sats,
                &cents,
                mappings,
                window_starts,
            )
        });
        Ok(Self { cohorts, stored })
    }

    #[inline(always)]
    pub fn push_block(&mut self, sats: CreationCohorts<Sats>, cents: CreationCohorts<Cents>) {
        self.stored.push_block(&sats, &cents);
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.stored.stored_vecs_mut().collect()
    }
}
