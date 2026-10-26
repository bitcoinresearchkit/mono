use bitview_cohort::AgeAggregateId;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::PartsPerMillionSigned32;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlock, LazyRatioPerBlock};
use brk_types::{Cents, CentsSigned, Version};

use crate::columns::Columns;

#[derive(Clone, Traversable)]
pub struct Unrealized {
    /// Unrealized profit of the cohort's unspent outputs: market value at the
    /// represented block minus creation-date value, summed where spot is above
    /// creation price.
    #[traversable(lead = "Unrealized profit: holders' paper gains, on coins worth more than they paid.")]
    pub profit: LazyFiatPerBlock<Cents>,
    /// Unrealized loss of the cohort's unspent outputs: creation-date value
    /// minus market value at the represented block, summed where spot is below
    /// creation price.
    #[traversable(lead = "Unrealized loss: holders' paper losses, on coins worth less than they paid.")]
    pub loss: LazyFiatPerBlock<Cents>,
    /// Net unrealized profit and loss of the cohort: unrealized profit minus
    /// unrealized loss.
    #[traversable(lead = "Net unrealized profit and loss: paper gains minus paper losses.")]
    pub net_pnl: LazyFiatPerBlock<CentsSigned>,
    /// Gross unrealized profit and loss: unrealized profit plus unrealized loss.
    #[traversable(lead = "Gross unrealized profit and loss: paper gains plus paper losses.")]
    pub gross_pnl: LazyFiatPerBlock<Cents>,
    /// Net unrealized profit/loss (NUPL): net unrealized profit and loss divided by the
    /// cohort's market cap: at most 1, with no lower bound (1 - 1/MVRV in exact arithmetic,
    /// though the two round and are undefined apart); undefined (null) when that market cap is
    /// zero.
    #[traversable(lead = "NUPL (net unrealized profit and loss): paper gains minus paper losses, as a share of the market cap. Near 1, almost all of it is profit; below 0, losses outweigh gains.")]
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
