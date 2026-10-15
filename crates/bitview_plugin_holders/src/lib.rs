#![doc = include_str!("../README.md")]

mod activity;
mod adjusted_sopr;
mod capital;
mod columns;
mod compute;
mod cost_basis;
mod data;
mod dependencies;
mod import;
mod live;
mod metrics;
mod part;
mod realized;
mod sources;
mod supply;
mod unrealized;

pub use dependencies::Dependencies;
pub use metrics::Metrics;

use bitview_cohort::AgeAggregate;
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, ReadableBoxedVec, Rw, StorageMode};

use cost_basis::CostBasisVecs;
use live::LiveState;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("holders"), Version::new(3));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
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
