use bitview_cohort::{CohortContext, CohortId, CreationCohorts};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazySpotValuePerBlock;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use crate::metrics::CreationSources;

#[derive(Traversable)]
pub struct SupplyByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: CreationCohorts<LazySpotValuePerBlock>,
    #[traversable(hidden)]
    pub stored: CreationSources<Sats, M>,
}

impl SupplyByCohort {
    pub fn import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let stored = CreationSources::import(db, &format!("{metric}_sats"), version)?;
        let cohorts = CreationCohorts::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let source = stored.get(cohort_id).expect("supported supply cohort");
            LazySpotValuePerBlock::from_sats_source(&name, version, source, mappings, spot_price)
        });

        Ok(Self { cohorts, stored })
    }

    pub fn get(&self, cohort_id: CohortId) -> Option<&LazySpotValuePerBlock> {
        self.cohorts.get(cohort_id)
    }

    #[inline(always)]
    pub fn push(&mut self, cohort_values: CreationCohorts<Sats>) {
        self.stored.push(&cohort_values);
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.stored.stored_vecs_mut().collect()
    }
}
