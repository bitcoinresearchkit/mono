use bitview_cohort::{CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::LazyFiatPerBlock;
use brk_types::{Cents, CentsSigned, Version};

use super::Sources;

#[derive(Clone, Traversable)]
pub struct UnrealizedMetrics {
    /// Current market value above creation-date value for profitable outputs.
    profit: LazyFiatPerBlock<Cents>,
    /// Creation-date value above current market value for losing outputs.
    loss: LazyFiatPerBlock<Cents>,
    /// Unrealized profit minus unrealized loss.
    net_pnl: LazyFiatPerBlock<CentsSigned>,
}

impl UnrealizedMetrics {
    pub(super) fn new(
        id: CohortId,
        version: Version,
        sources: &Sources,
        mappings: &Mappings,
    ) -> Self {
        let name = |metric| CohortContext::Utxo.metric_name(id, metric);
        let loss = LazyFiatPerBlock::from_cents_source(
            &name("unrealized_loss"),
            version,
            &sources.unrealized_loss,
            mappings,
        );
        Self {
            profit: LazyFiatPerBlock::from_cents_source(
                &name("unrealized_profit"),
                version,
                &sources.unrealized_profit,
                mappings,
            ),
            loss,
            net_pnl: LazyFiatPerBlock::from_cents_source(
                &name("net_unrealized_pnl"),
                version,
                &sources.unrealized_net_pnl,
                mappings,
            ),
        }
    }
}
