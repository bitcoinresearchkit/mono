use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::{Difficulty, Hashrate};
use bitview_vecs::LazyPerBlock;
use brk_types::{Height, Sats};
use vecdb::PcoVec;

pub struct Dependencies<'a> {
    pub mappings: &'a MappingsVecs,
    pub difficulty_hashrate: &'a LazyPerBlock<Hashrate, Difficulty>,
    pub coinbase_value: &'a PcoVec<Height, Sats>,
    pub total_fees: &'a PcoVec<Height, Sats>,
    pub price: &'a PriceVecs,
}
