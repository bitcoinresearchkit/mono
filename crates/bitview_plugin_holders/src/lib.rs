#![doc = include_str!("../README.md")]

mod activity;
mod adjusted_sopr;
mod columns;
mod compute;
mod cost_basis;
mod data;
mod density_sources;
mod dependencies;
mod has;
mod import;
mod live;
mod metrics;
mod outputs;
mod ratios;
mod realized;
mod relative;
mod sources;
mod supply;
mod unrealized;
mod unrealized_data;

pub use dependencies::Dependencies;
pub use has::HasHolders;
pub use metrics::Metrics;

use bitview_cohort::AgeAggregate;
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, ReadableBoxedVec, Rw, StorageMode};

use cost_basis::CostBasisVecs;
use live::LiveState;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("holders"), Version::ONE);
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    #[traversable(flatten)]
    pub cohorts: AgeAggregate<Metrics<M>>,
    #[traversable(hidden)]
    cost_basis: Box<CostBasisVecs<M>>,
    #[traversable(skip)]
    all_supply: ReadableBoxedVec<Height, Sats>,
    #[traversable(skip)]
    all_market_cap: ReadableBoxedVec<Height, Cents>,
    live: M::WriteOnly<Option<LiveState>>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
