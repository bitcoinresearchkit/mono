use crate::groups::UtxoGroups;
use crate::sources::UtxoSources;
use crate::values::UtxoValues;
use bitview_cohort::{CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazySpotValuePerBlock;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, ReadableCloneableVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct SupplyTotal<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UtxoGroups<LazySpotValuePerBlock>,
    #[traversable(hidden)]
    pub stored: UtxoSources<Sats, M>,
    #[traversable(skip)]
    all_supply: ReadableBoxedVec<Height, Sats>,
}

impl SupplyTotal {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let stored = UtxoSources::forced_import(db, "supply_sats", version)?;
        let all_supply = all_supply.read_only_boxed_clone();
        let cohorts = UtxoGroups::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "supply");
            let source = stored.get(cohort_id).expect("UTXO supply source");
            LazySpotValuePerBlock::from_sats_source(&name, version, source, mappings, spot_price)
        });
        Ok(Self {
            cohorts,
            stored,
            all_supply,
        })
    }

    pub fn min_len(&self) -> usize {
        self.stored.min_len()
    }

    pub fn get(&self, cohort_id: CohortId) -> Option<&LazySpotValuePerBlock> {
        self.cohorts.get(cohort_id)
    }

    pub fn all_supply(&self) -> &ReadableBoxedVec<Height, Sats> {
        &self.all_supply
    }

    #[inline(always)]
    pub fn push(&mut self, cohort_values: UtxoValues<Sats>) {
        self.stored.push(cohort_values);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored.stored_vecs_mut()
    }
}
