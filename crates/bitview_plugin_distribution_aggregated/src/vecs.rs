use crate::{STORAGE, cost_basis::CostBasisVecs, live::LiveState, metrics::Metrics};
use bitview_cohort::AgeAggregate;
use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_distribution_common::AllChainSources;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_plugin_price::Vecs as Price;
use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyIndexedVec, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{Cents, Height, Sats};
use vecdb::{BinaryTransform, Database, ReadableBoxedVec, ReadableCloneableVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) db: Database,
    pub cohorts: AgeAggregate<Metrics<M>>,
    #[traversable(hidden)]
    pub(crate) cost_basis: Box<CostBasisVecs<M>>,
    #[traversable(skip)]
    pub(crate) all_supply: ReadableBoxedVec<Height, Sats>,
    #[traversable(skip)]
    pub(crate) all_market_cap: ReadableBoxedVec<Height, Cents>,
    pub(crate) live: M::WriteOnly<Option<LiveState>>,
}
impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        price: &Price,
        supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();
        let spot = price.spot.cents.height.read_only_boxed_clone();
        let cost_basis = CostBasisVecs::forced_import(&db, version, mappings)?;
        let cohorts = AgeAggregate::try_from_fn(|id| {
            Metrics::import(
                &db,
                id,
                version,
                mappings,
                windows,
                &spot,
                id.select(&cost_basis.cohorts).clone(),
            )
        })?;
        let all_market_cap = LazyIndexedVec::new(
            "aggregate_market_cap_source",
            version,
            supply,
            &spot,
            |_, sats, price| SatsToCents::apply(sats, price),
        )
        .read_only_boxed_clone();
        STORAGE.finalize_database(&db)?;
        Ok(Self {
            db,
            cohorts,
            cost_basis,
            all_supply: supply.read_only_boxed_clone(),
            all_market_cap,
            live: None,
        })
    }
    pub fn all_chain_sources(&self) -> AllChainSources {
        AllChainSources::new(&self.all_supply, &self.all_market_cap)
    }
    pub fn all_supply(&self) -> &ReadableBoxedVec<Height, Sats> {
        &self.all_supply
    }
}
