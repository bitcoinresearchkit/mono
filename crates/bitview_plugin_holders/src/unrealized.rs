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
    /// Net unrealized profit/loss (NUPL): net unrealized profit and loss divided by the
    /// cohort's market cap.
    pub nupl: LazyRatioPerBlock<PartsPerMillionSigned32>,
}
impl Unrealized {
    pub(crate) fn new(id: AgeAggregateId, v: Version, c: &Columns, mappings: &Mappings) -> Self {
        let fiat = |metric: &str, source| {
            LazyFiatPerBlock::from_cents_source(&id.metric_name(metric), v, source, mappings)
        };
        Self {
            profit: fiat("unrealized_profit", &c.unrealized_profit),
            loss: fiat("unrealized_loss", &c.unrealized_loss),
            net_pnl: LazyFiatPerBlock::from_cents_source(
                &id.metric_name("net_unrealized_pnl"),
                v,
                &c.unrealized_net_pnl,
                mappings,
            ),
            gross_pnl: fiat("gross_unrealized_pnl", &c.unrealized_gross_pnl),
            nupl: LazyRatioPerBlock::from_height_source(
                &id.metric_name("nupl"),
                v,
                &c.nupl,
                mappings,
            ),
        }
    }
}
