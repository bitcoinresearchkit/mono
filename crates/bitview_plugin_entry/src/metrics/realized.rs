use bitview_cohort::{CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlockCumulativeWithSums, LazyWindowStartVec, shapes::Windows};
use brk_types::{Cents, CentsSigned, Version};

use super::Sources;

#[derive(Clone, Traversable)]
pub struct RealizedMetrics {
    /// Profit realized by outputs spent from this cohort.
    profit: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Loss realized by outputs spent from this cohort.
    loss: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Realized profit minus realized loss.
    net_pnl: LazyFiatPerBlockCumulativeWithSums<CentsSigned>,
    /// Creation-time value of the outputs spent.
    value_destroyed: LazyFiatPerBlockCumulativeWithSums<Cents>,
}

impl RealizedMetrics {
    pub(super) fn new(
        id: CohortId,
        version: Version,
        sources: &Sources,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let name = |metric| CohortContext::Utxo.metric_name(id, metric);
        let flow = |metric, source| {
            LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &name(metric),
                version,
                source,
                mappings,
                windows,
            )
        };
        Self {
            profit: flow(
                "realized_profit",
                sources.realized_profit.cumulative_source(),
            ),
            loss: flow("realized_loss", sources.realized_loss.cumulative_source()),
            net_pnl: LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &name("net_realized_pnl"),
                version,
                sources.realized_net_pnl.cumulative_source(),
                mappings,
                windows,
            ),
            value_destroyed: flow(
                "value_destroyed",
                sources.value_destroyed.cumulative_source(),
            ),
        }
    }
}
