#![allow(clippy::type_complexity)]

mod addr;
mod chain_counts;
mod compute;
mod dependencies;
mod has;
mod height;
mod height_lookup;
mod identity;
mod import;
mod lookback;
mod resolution;
mod timestamp;
mod tx_index;
mod views;

pub use dependencies::Dependencies;
pub use has::HasMappings;
pub use height_lookup::HeightMap;
pub use lookback::LookbackVecs;
pub use views::{LazyCumulativeIndexVec, LazyIndexCountVec, RangeMapLookupVec};

use std::ops::Deref;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{
    Day1, Day3, Epoch, Halving, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3, Month6,
    TxInIndex, TxOutIndex, Week1, Year1, Year10,
};
use bitview_traversable::Traversable;
use bitview_vecs::IndexSources;
use brk_types::{Height, OutPoint, OutputType, TxIndex, Version};
use vecdb::{Database, Rw, StorageMode};

use addr::{OutputVecs, Vecs as AddressesVecs};
use chain_counts::ChainCounts;
use height::Vecs as HeightVecs;
use height_lookup::HeightLookup;
use identity::IdentityVecs;
use resolution::{DatedResolutionVecs, ResolutionVecs};
use timestamp::Timestamps;
use tx_index::Vecs as TxIndexVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("mappings"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
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
    addresses: AddressesVecs,
    outputs: OutputVecs,
    pub height: HeightVecs,
    pub epoch: ResolutionVecs<Epoch, M>,
    halving: ResolutionVecs<Halving, M>,
    minute10: ResolutionVecs<Minute10, M>,
    minute30: ResolutionVecs<Minute30, M>,
    hour1: ResolutionVecs<Hour1, M>,
    hour4: ResolutionVecs<Hour4, M>,
    hour12: ResolutionVecs<Hour12, M>,
    pub day1: DatedResolutionVecs<Day1, M>,
    day3: DatedResolutionVecs<Day3, M>,
    week1: DatedResolutionVecs<Week1, M>,
    month1: DatedResolutionVecs<Month1, M>,
    month3: DatedResolutionVecs<Month3, M>,
    month6: DatedResolutionVecs<Month6, M>,
    year1: DatedResolutionVecs<Year1, M>,
    year10: DatedResolutionVecs<Year10, M>,
    pub tx_index: TxIndexVecs,
    /// Global zero-based transaction-input index in canonical blockchain order. At `txin_index`,
    /// returns the input index itself.
    txin_index: IdentityVecs<TxInIndex, OutPoint>,
    /// Global zero-based transaction-output index in canonical blockchain order. At
    /// `txout_index`, returns the output index itself.
    txout_index: IdentityVecs<TxOutIndex, OutputType>,
    pub timestamp: Timestamps<M>,
    // First block height inside each trailing duration (from the monotonic timestamps):
    // window starts for other plugins, not a public series.
    #[traversable(hidden)]
    pub lookback: LookbackVecs,
}

impl<M: StorageMode> Deref for Vecs<M> {
    type Target = IndexSources;

    fn deref(&self) -> &Self::Target {
        &self.sources
    }
}

impl Vecs {
    pub fn transaction_count_source(&self) -> LazyCumulativeIndexVec<Height, TxIndex> {
        self.chain_counts.transaction.clone()
    }

    pub fn input_count_source(&self) -> LazyCumulativeIndexVec<Height, TxInIndex> {
        self.chain_counts.input.clone()
    }

    pub fn output_count_source(&self) -> LazyCumulativeIndexVec<Height, TxOutIndex> {
        self.chain_counts.output.clone()
    }
}
