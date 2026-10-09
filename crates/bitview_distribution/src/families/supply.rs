use bitview_cohort::{AgeRange, CohortContext, CohortGroup, cohort_group::Creation};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CohortSources, LazySpotValuePerBlock};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, Database, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, Rw, StorageMode,
};

/// Sats per cohort, stored as `{metric}_sats` and valued at the block's spot price.
#[derive(Traversable)]
pub struct SupplyByCohort<G: CohortGroup, M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: G::Of<LazySpotValuePerBlock>,
    #[traversable(hidden)]
    pub stored: CohortSources<G, Sats, M>,
}

impl<G: CohortGroup> SupplyByCohort<G> {
    pub fn import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let stored = CohortSources::import(db, &format!("{metric}_sats"), version)?;
        let cohorts = G::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let source = stored.get(cohort_id).expect("supply cohort source");
            LazySpotValuePerBlock::from_sats_source(&name, version, source, mappings, spot_price)
        });
        Ok(Self { cohorts, stored })
    }

    #[inline(always)]
    pub fn push(&mut self, supplies: &G::Of<Sats>) {
        self.stored.push(supplies);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored.stored_vecs_mut()
    }
}

/// Every cohort's supply, next to the total supply it divides.
#[derive(Deref, DerefMut, Traversable)]
pub struct SupplyTotal<G: CohortGroup, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    supply: SupplyByCohort<G, M>,
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
        Ok(Self {
            supply: SupplyByCohort::import(db, "supply", version, mappings, spot_price)?,
            all_supply: all_supply.read_only_boxed_clone(),
        })
    }

    pub fn all_supply(&self) -> &ReadableBoxedVec<Height, Sats> {
        &self.all_supply
    }
}

impl SupplyTotal<Creation> {
    pub fn age_supplies(&self) -> AgeRange<&impl ReadableVec<Height, Sats>> {
        AgeRange::from_fn(|id| &id.select(&self.cohorts.age).sats.height)
    }
}
