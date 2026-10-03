use bitview_cohort::{AgeRange, CohortContext, CohortId, CreationCohorts, UTXOCoreValues};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazySpotValuePerBlock;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{
    AnyStoredVec, Database, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, Rw, StorageMode,
};

use crate::metrics::CreationSources;

#[derive(Traversable)]
pub struct SupplyTotal<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: CreationCohorts<LazySpotValuePerBlock>,
    #[traversable(hidden)]
    pub stored: CreationSources<Sats, M>,
    #[traversable(skip)]
    all_supply: ReadableBoxedVec<Height, Sats>,
}

impl SupplyTotal {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_sats: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let stored = CreationSources::forced_import(db, "supply_sats", version)?;
        let all_supply = all_sats.read_only_boxed_clone();
        let cohorts = CreationCohorts::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "supply");
            let source = stored.get(cohort_id).expect("total-supply cohort source");
            LazySpotValuePerBlock::from_sats_source(&name, version, source, mappings, spot_price)
        });

        Ok(Self {
            cohorts,
            stored,
            all_supply,
        })
    }

    pub fn age_supplies(&self) -> AgeRange<&impl ReadableVec<Height, Sats>> {
        AgeRange::from_fn(|id| &id.select(&self.cohorts.age).sats.height)
    }

    pub fn get(&self, cohort_id: CohortId) -> Option<&LazySpotValuePerBlock> {
        self.cohorts.get(cohort_id)
    }

    pub fn all_supply(&self) -> &ReadableBoxedVec<Height, Sats> {
        &self.all_supply
    }

    #[inline(always)]
    pub fn push(&mut self, cohort_values: UTXOCoreValues<Sats>) {
        self.stored.push(cohort_values);
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.stored.collect_vecs_mut()
    }
}
