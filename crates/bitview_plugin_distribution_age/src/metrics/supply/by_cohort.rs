use bitview_cohort::{CohortContext, CohortId, UTXOCoreValues, UTXOGroupsWithoutAmountOrType};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazySpotValuePerBlock;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use crate::metrics::UTXOCoreSources;

#[derive(Traversable)]
pub struct SupplyByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<LazySpotValuePerBlock>,
    #[traversable(hidden)]
    pub stored: UTXOCoreSources<Sats, M>,
}

impl SupplyByCohort {
    pub fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let stored = UTXOCoreSources::forced_import(db, &format!("{metric}_sats"), version)?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let source = stored.get(cohort_id).expect("supported supply cohort");
            LazySpotValuePerBlock::from_sats_source(&name, version, source, mappings, spot_price)
        });

        Ok(Self { cohorts, stored })
    }

    pub fn get(&self, cohort_id: CohortId) -> Option<&LazySpotValuePerBlock> {
        self.cohorts.get(cohort_id)
    }

    pub fn min_len(&self) -> usize {
        self.stored.min_len()
    }

    #[inline(always)]
    pub fn push(&mut self, cohort_values: UTXOCoreValues<Sats>) {
        self.stored.push(cohort_values);
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.stored.collect_vecs_mut()
    }
}
