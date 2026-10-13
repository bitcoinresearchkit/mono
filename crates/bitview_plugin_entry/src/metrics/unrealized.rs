use bitview_cohort::{CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::PartsPerMillionSigned32;
use bitview_transforms::{Quotient, SatsToCents};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlock, LazyIndexedVec, LazyRatioPerBlock};
use brk_types::{Cents, CentsSigned, Height, Version};
use vecdb::{BinaryTransform, ReadableBoxedVec};

use super::Sources;

#[derive(Clone, Traversable)]
pub struct UnrealizedMetrics {
    /// Current market value above creation-date value for profitable outputs.
    profit: LazyFiatPerBlock<Cents>,
    /// Creation-date value above current market value for losing outputs.
    loss: LazyFiatPerBlock<Cents>,
    /// Unrealized profit minus unrealized loss.
    net_pnl: LazyFiatPerBlock<CentsSigned>,
    /// Net unrealized profit/loss as a share of this cohort's own market cap (its supply at spot).
    /// Undefined (null) while that market cap is zero.
    nupl: LazyRatioPerBlock<PartsPerMillionSigned32>,
}

impl UnrealizedMetrics {
    pub(super) fn new(
        id: CohortId,
        version: Version,
        sources: &Sources,
        mappings: &Mappings,
        prices: &ReadableBoxedVec<Height, Cents>,
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
            nupl: LazyRatioPerBlock::from_ratio::<CentsSigned, Cents, Quotient<_>>(
                &name("nupl"),
                version,
                &sources.unrealized_net_pnl,
                &LazyIndexedVec::new(
                    &name("market_cap_source"),
                    version,
                    &sources.supply,
                    prices,
                    |_, supply, spot| SatsToCents::apply(supply, spot),
                ),
                mappings,
            ),
        }
    }
}
