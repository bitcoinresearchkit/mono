#![allow(clippy::type_complexity)]

mod addr;
mod chain_counts;
mod compute;
mod dependencies;
mod has;
mod height;
mod height_lookup;
mod import;
mod resolution;
mod timestamp;
mod tx_index;
mod txin_index;
mod txout_index;
mod views;

pub use dependencies::Dependencies;
pub use has::HasMappings;
pub use height_lookup::HeightMap;
pub use views::{LazyCumulativeIndexVec, LazyIndexCountVec, RangeMapLookupVec};

use std::ops::Deref;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{
    Day1, Day3, Epoch, Halving, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3, Month6,
    StoredU64, TxInIndex, TxOutIndex, Week1, Year1, Year10,
};
use bitview_traversable::Traversable;
use bitview_vecs::IndexSources;
use brk_types::{Height, TxIndex, Version};
use vecdb::{Database, ReadableBoxedVec, Rw, StorageMode};

use addr::Vecs as AddrVecs;
use chain_counts::ChainCounts;
use height::Vecs as HeightVecs;
use height_lookup::HeightLookup;
use resolution::{DatedResolutionVecs, ResolutionVecs};
use timestamp::Timestamps;
use tx_index::Vecs as TxIndexVecs;
use txin_index::Vecs as TxInIndexVecs;
use txout_index::Vecs as TxOutIndexVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("mappings"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    chain_counts: M::WriteOnly<ChainCounts>,
    #[traversable(skip)]
    sources: IndexSources,
    #[traversable(skip)]
    pub tx_heights: HeightLookup<TxIndex>,
    #[traversable(skip)]
    pub output_heights: HeightLookup<TxOutIndex>,
    addr: AddrVecs,
    pub height: HeightVecs,
    pub epoch: ResolutionVecs<Epoch>,
    halving: ResolutionVecs<Halving>,
    minute10: ResolutionVecs<Minute10>,
    minute30: ResolutionVecs<Minute30>,
    hour1: ResolutionVecs<Hour1>,
    hour4: ResolutionVecs<Hour4>,
    hour12: ResolutionVecs<Hour12>,
    pub day1: DatedResolutionVecs<Day1>,
    day3: DatedResolutionVecs<Day3>,
    week1: DatedResolutionVecs<Week1>,
    month1: DatedResolutionVecs<Month1>,
    month3: DatedResolutionVecs<Month3>,
    month6: DatedResolutionVecs<Month6>,
    year1: DatedResolutionVecs<Year1>,
    year10: DatedResolutionVecs<Year10>,
    pub tx_index: TxIndexVecs,
    txin_index: TxInIndexVecs,
    txout_index: TxOutIndexVecs,
    pub timestamp: Timestamps<M>,
}

impl<M: StorageMode> Deref for Vecs<M> {
    type Target = IndexSources;

    fn deref(&self) -> &Self::Target {
        &self.sources
    }
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

impl Vecs {
    pub fn transaction_count_source(&self) -> LazyCumulativeIndexVec<Height, TxIndex> {
        self.chain_counts.transaction_source()
    }

    pub fn input_count_source(&self) -> LazyCumulativeIndexVec<Height, TxInIndex> {
        self.chain_counts.input_source()
    }

    pub fn output_count(&self) -> ReadableBoxedVec<Height, StoredU64> {
        self.chain_counts.output()
    }

    pub fn output_count_source(&self) -> LazyCumulativeIndexVec<Height, TxOutIndex> {
        self.chain_counts.output_source()
    }
}
