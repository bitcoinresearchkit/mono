use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{PartsPerMillionSigned64, PriceRatio};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlockCumulativeWithSums, LazyFiatPerBlockWithDeltas, LazyPerBlock,
    LazyPriceWithRatioPerBlock, LazyRatioPerBlock, LazyWindowStartVec, Price,
};
use brk_types::{Cents, CentsSigned, Height, Version};
use vecdb::ReadableBoxedVec;

use crate::columns::Columns;

#[derive(Clone, Traversable)]
pub struct Realized {
    pub cap: LazyFiatPerBlockWithDeltas<Cents, CentsSigned, PartsPerMillionSigned64>,
    /// Realized price: acquisition cost of remaining coins divided by their supply.
    pub price: Price<LazyPerBlock<Cents>>,
    pub capitalized_price: LazyPriceWithRatioPerBlock,
    pub profit: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub loss: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub net_pnl: LazyFiatPerBlockCumulativeWithSums<CentsSigned>,
    pub value_destroyed: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub gross_pnl: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub peak_regret: LazyFiatPerBlockCumulativeWithSums<Cents>,
    pub mvrv: LazyRatioPerBlock<PriceRatio>,
}
impl Realized {
    pub(crate) fn new(
        id: AgeAggregateId,
        v: Version,
        c: &Columns,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
    ) -> Self {
        let cap = LazyFiatPerBlockWithDeltas::from_cents_source(
            &id.metric_name("realized_cap"),
            v,
            &c.cap,
            v,
            mappings,
            windows,
        );
        let price =
            Price::from_height_source(&id.metric_name("realized_price"), v, &c.price, mappings);
        let capitalized_price = LazyPriceWithRatioPerBlock::from_height_source(
            &id.metric_name("capitalized_price"),
            v,
            &c.capitalized_price,
            mappings,
            spot,
        );
        let profit = LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
            &id.metric_name("realized_profit"),
            v,
            &c.profit,
            mappings,
            windows,
        );
        let loss = LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
            &id.metric_name("realized_loss"),
            v,
            &c.loss,
            mappings,
            windows,
        );
        let net_pnl = LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
            &id.metric_name("net_realized_pnl"),
            v,
            &c.net_pnl,
            mappings,
            windows,
        );
        let value_destroyed = LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
            &id.metric_name("value_destroyed"),
            v,
            &c.value_destroyed,
            mappings,
            windows,
        );
        let gross_pnl = LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
            &id.metric_name("realized_gross_pnl"),
            v,
            &c.gross_pnl,
            mappings,
            windows,
        );
        let peak_regret = LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
            &id.metric_name("realized_peak_regret"),
            v,
            &c.peak_regret,
            mappings,
            windows,
        );
        let mvrv = LazyRatioPerBlock::from_price_source(
            &id.metric_name("mvrv"),
            v,
            &price.cents.height,
            spot,
            mappings,
        );
        Self {
            cap,
            price,
            capitalized_price,
            profit,
            loss,
            net_pnl,
            value_destroyed,
            gross_pnl,
            peak_regret,
            mvrv,
        }
    }
}
