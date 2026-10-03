use bitview_cohort::{AgeRange, CohortContext, CohortGroup, cohort_group::Creation};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CohortSources, LazySpotValuePerBlock};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{
    AnyStoredVec, Database, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, Rw, StorageMode,
};

#[derive(Traversable)]
pub struct SupplyTotal<G: CohortGroup, M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: G::Of<LazySpotValuePerBlock>,
    #[traversable(hidden)]
    pub stored: CohortSources<G, Sats, M>,
    #[traversable(skip)]
    all_supply: ReadableBoxedVec<Height, Sats>,
}

impl<G: CohortGroup> SupplyTotal<G> {
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let stored = CohortSources::import(db, "supply_sats", version)?;
        let cohorts = G::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "supply");
            let source = stored.get(cohort_id).expect("total-supply cohort source");
            LazySpotValuePerBlock::from_sats_source(&name, version, source, mappings, spot_price)
        });
        Ok(Self {
            cohorts,
            stored,
            all_supply: all_supply.read_only_boxed_clone(),
        })
    }

    pub fn all_supply(&self) -> &ReadableBoxedVec<Height, Sats> {
        &self.all_supply
    }

    #[inline(always)]
    pub fn push(&mut self, supplies: &G::Of<Sats>) {
        self.stored.push(supplies);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored.stored_vecs_mut()
    }
}

impl SupplyTotal<Creation> {
    pub fn age_supplies(&self) -> AgeRange<&impl ReadableVec<Height, Sats>> {
        AgeRange::from_fn(|id| &id.select(&self.cohorts.age).sats.height)
    }
}
