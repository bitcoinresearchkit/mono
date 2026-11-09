#![doc = include_str!("../README.md")]

mod accounting;
mod activity;
mod adjusted_sopr;
mod age_aggregate;
mod capital;
mod columns;
mod compute;
mod cost_basis;
mod dependencies;
mod import;
mod live;
mod metrics;
mod part;
mod realized;
mod section;
mod sources;
mod supply;
mod unrealized;

pub use dependencies::Dependencies;
pub use metrics::{All, Kind, Metrics, Old, Young};

use bitview_cohort::AgeAggregate;
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, ReadableBoxedVec, ReadableCloneableVec, Rw, StorageMode};

use columns::Columns;
use cost_basis::CostBasisVecs;
use live::LiveState;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("holders"), Version::new(5));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    #[traversable(flatten)]
    pub cohorts: AgeAggregate<Metrics<Young, M>, Metrics<All, M>, Metrics<Old, M>>,
    /// What the block loop writes for each group, its series' sources.
    #[traversable(hidden)]
    columns: AgeAggregate<Columns<M>>,
    #[traversable(hidden)]
    cost_basis: Box<CostBasisVecs<M>>,
    #[traversable(skip)]
    all_supply: ReadableBoxedVec<Height, Sats>,
    #[traversable(skip)]
    all_market_cap: ReadableBoxedVec<Height, Cents>,
    live: M::WriteOnly<Option<LiveState>>,
}

impl<M: StorageMode> Vecs<M> {
    /// All-chain creation price weighted by each output's dollar value at creation, in cents.
    pub fn all_capitalized_price(&self) -> ReadableBoxedVec<Height, Cents> {
        self.cohorts
            .all
            .cost_basis
            .by_dollar
            .avg
            .cents
            .height
            .read_only_boxed_clone()
    }
}
