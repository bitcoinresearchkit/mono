use crate::columns::Columns;
use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyRollingDeltasAmountFromHeight, LazySpotValuePerBlock, LazyWindowStartVec};
use brk_types::{Cents, Height, PartsPerMillionSigned64, Sats, SatsSigned, Version};
use vecdb::ReadableBoxedVec;

#[derive(Clone, Traversable)]
pub struct Supply {
    /// Total unspent supply.
    pub total: LazySpotValuePerBlock,
    /// Supply in profit: acquisition price is at or below the current spot price.
    pub in_profit: LazySpotValuePerBlock,
    /// Supply in loss: acquisition price is above the current spot price.
    pub in_loss: LazySpotValuePerBlock,
    pub delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
}
impl Supply {
    pub(crate) fn new(
        id: AgeAggregateId,
        v: Version,
        c: &Columns,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
    ) -> Self {
        let total = LazySpotValuePerBlock::from_sats_source(
            &id.metric_name("supply"),
            v,
            &c.supply,
            mappings,
            spot,
        );
        let in_profit = LazySpotValuePerBlock::from_sats_source(
            &id.metric_name("supply_in_profit"),
            v,
            &c.supply_profit,
            mappings,
            spot,
        );
        let in_loss = LazySpotValuePerBlock::from_sats_source(
            &id.metric_name("supply_in_loss"),
            v,
            &c.supply_loss,
            mappings,
            spot,
        );
        let delta = LazyRollingDeltasAmountFromHeight::new(
            &id.metric_name("supply_delta"),
            v,
            &c.supply,
            windows,
            mappings,
        );
        Self {
            total,
            in_profit,
            in_loss,
            delta,
        }
    }
}
