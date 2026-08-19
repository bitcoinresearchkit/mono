use bitview_compute::{LazyPerBlock, LazyPercentPerBlock};
use bitview_plugin_indexer::Indexer;
use brk_types::{PartsPerMillionSigned64, StoredF64};

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub price: &'a bitview_plugin_price::Vecs,
    pub blocks: &'a bitview_plugin_blocks::Vecs,
    pub inflation_rate: &'a LazyPercentPerBlock<PartsPerMillionSigned64>,
    pub velocity_native: &'a LazyPerBlock<StoredF64>,
    pub velocity_fiat: &'a LazyPerBlock<StoredF64>,
    pub distribution: &'a bitview_plugin_distribution::Vecs,
}
