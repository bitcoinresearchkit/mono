use bitview_cohort::AgeAggregateId;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::PartsPerMillionSigned32;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlock, LazyRatioPerBlock};
use brk_types::{Cents, CentsSigned, Version};

use crate::columns::Columns;

#[derive(Clone, Traversable)]
pub struct Unrealized {
    pub profit: LazyFiatPerBlock<Cents>,
    pub loss: LazyFiatPerBlock<Cents>,
    pub net_pnl: LazyFiatPerBlock<CentsSigned>,
    pub gross_pnl: LazyFiatPerBlock<Cents>,
    pub invested_capital_in_profit: LazyFiatPerBlock<Cents>,
    pub invested_capital_in_loss: LazyFiatPerBlock<Cents>,
    pub pain_index: LazyFiatPerBlock<Cents>,
    pub greed_index: LazyFiatPerBlock<Cents>,
    pub net_sentiment: LazyFiatPerBlock<CentsSigned>,
    pub nupl: LazyRatioPerBlock<PartsPerMillionSigned32>,
}
impl Unrealized {
    pub(crate) fn new(id: AgeAggregateId, v: Version, c: &Columns, mappings: &Mappings) -> Self {
        let profit = LazyFiatPerBlock::from_cents_source(
            &id.metric_name("unrealized_profit"),
            v,
            &c.unrealized_profit,
            mappings,
        );
        let loss = LazyFiatPerBlock::from_cents_source(
            &id.metric_name("unrealized_loss"),
            v,
            &c.unrealized_loss,
            mappings,
        );
        let net_pnl = LazyFiatPerBlock::from_cents_source(
            &id.metric_name("net_unrealized_pnl"),
            v,
            &c.unrealized_net_pnl,
            mappings,
        );
        let gross_pnl = LazyFiatPerBlock::from_cents_source(
            &id.metric_name("unrealized_gross_pnl"),
            v,
            &c.unrealized_gross_pnl,
            mappings,
        );
        let invested_capital_in_profit = LazyFiatPerBlock::from_cents_source(
            &id.metric_name("invested_capital_in_profit"),
            v,
            &c.invested_profit,
            mappings,
        );
        let invested_capital_in_loss = LazyFiatPerBlock::from_cents_source(
            &id.metric_name("invested_capital_in_loss"),
            v,
            &c.invested_loss,
            mappings,
        );
        let pain_index = LazyFiatPerBlock::from_cents_source(
            &id.metric_name("pain_index"),
            v,
            &c.pain,
            mappings,
        );
        let greed_index = LazyFiatPerBlock::from_cents_source(
            &id.metric_name("greed_index"),
            v,
            &c.greed,
            mappings,
        );
        let net_sentiment = LazyFiatPerBlock::from_cents_source(
            &id.metric_name("net_sentiment"),
            v,
            &c.sentiment,
            mappings,
        );
        let nupl =
            LazyRatioPerBlock::from_height_source(&id.metric_name("nupl"), v, &c.nupl, mappings);
        Self {
            profit,
            loss,
            net_pnl,
            gross_pnl,
            invested_capital_in_profit,
            invested_capital_in_loss,
            pain_index,
            greed_index,
            net_sentiment,
            nupl,
        }
    }
}
