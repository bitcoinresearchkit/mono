use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::PartsPerMillionSigned64;
use bitview_urpd::StoredReplayInputs;
use bitview_vecs::{LazyPercentPerBlock, LazyRollingSumAmountFromHeight};

#[derive(Clone, Copy)]
pub struct Dependencies<'a> {
    pub urpd: StoredReplayInputs<'a>,
    pub mappings: &'a MappingsVecs,
    pub price: &'a PriceVecs,
    pub inflation_rate: &'a LazyPercentPerBlock<PartsPerMillionSigned64>,
    /// The year's transfer volume, the cointime-adjusted velocity's numerator.
    pub volume_1y: &'a LazyRollingSumAmountFromHeight,
    pub holders: &'a HoldersVecs,
    pub age: &'a AgeVecs,
}
