use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_distribution_age::Vecs as AgeVecs;
use bitview_plugin_distribution_aggregated::Vecs as AggregatedVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_urpd::ReplayInputs;
use bitview_vecs::{LazyPerBlock, LazyPercentPerBlock};
use brk_types::{PartsPerMillionSigned64, StoredF64};

#[derive(Clone, Copy)]
pub struct Dependencies<'a> {
    pub urpd: ReplayInputs<'a>,
    pub indexer: &'a Indexer,
    pub price: &'a PriceVecs,
    pub blocks: &'a BlocksVecs,
    pub inflation_rate: &'a LazyPercentPerBlock<PartsPerMillionSigned64>,
    pub velocity_native: &'a LazyPerBlock<StoredF64>,
    pub velocity_fiat: &'a LazyPerBlock<StoredF64>,
    pub distribution_aggregated: &'a AggregatedVecs,
    pub distribution_age: &'a AgeVecs,
}
