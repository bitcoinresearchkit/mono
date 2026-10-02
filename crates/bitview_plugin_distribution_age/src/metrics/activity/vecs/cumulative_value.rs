use bitview_cohort::{CohortContext, CreationCohorts, UTXOCoreValues};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyValuePerBlockCumulativeRolling, LazyWindowStartVec, SatsCents};
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::metrics::CumulativeCreationValueSources;

#[derive(Traversable)]
pub struct CumulativeValueByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// UTXO groups and spent output value grouped by the spending address's
    /// balance immediately before the spend.
    pub cohorts: CreationCohorts<LazyValuePerBlockCumulativeRolling>,
    #[traversable(hidden)]
    pub stored: CumulativeCreationValueSources<M>,
}

impl CumulativeValueByCohort {
    pub fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = CumulativeCreationValueSources::forced_import(
            db,
            &format!("{metric}_cumulative"),
            version,
        )?;
        let cohorts = CreationCohorts::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let SatsCents { sats, cents } = stored
                .sources(cohort_id, &name, version)
                .expect("supported stored value cohort");
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
    pub fn push_block(&mut self, sats: UTXOCoreValues<Sats>, cents: UTXOCoreValues<Cents>) {
        self.stored.push_block(sats, cents);
    }

    pub fn min_len(&self) -> usize {
        self.stored.min_len()
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.stored.collect_vecs_mut()
    }
}
