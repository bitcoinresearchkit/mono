use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::{PartsPerMillionSigned64, Ratio64};
use bitview_urpd::ReplayInputs;
use bitview_vecs::{LazyPerBlock, LazyPercentPerBlock};

#[derive(Clone, Copy)]
pub struct Dependencies<'a> {
    pub urpd: ReplayInputs<'a>,
    pub indexer: &'a Indexer,
    pub price: &'a PriceVecs,
    pub blocks: &'a BlocksVecs,
    pub inflation_rate: &'a LazyPercentPerBlock<PartsPerMillionSigned64>,
    pub velocity_native: &'a LazyPerBlock<Ratio64>,
    pub velocity_fiat: &'a LazyPerBlock<Ratio64>,
    pub holders: &'a HoldersVecs,
    pub age: &'a AgeVecs,
}
