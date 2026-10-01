use bitview_cohort::{CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_transforms::{MvrvToNupl, NegCentsUnsignedToDollars};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock, LazyRatioPerBlock};
use brk_types::{Cents, CentsSigned, Dollars, PartsPerMillionSigned32, PriceRatio, Version};

use super::Sources;

#[derive(Clone, Traversable)]
pub struct UnrealizedMetrics {
    /// Current market value above creation-date value for profitable outputs.
    pub profit: LazyFiatPerBlock<Cents>,
    /// Creation-date value above current market value for losing outputs.
    pub loss: LazyFiatPerBlock<Cents>,
    /// Unrealized profit minus unrealized loss.
    pub net_pnl: LazyFiatPerBlock<CentsSigned>,
    /// Net unrealized profit/loss as a share of this cohort's own market cap.
    pub nupl: LazyRatioPerBlock<PartsPerMillionSigned32, PriceRatio>,
    #[traversable(wrap = "loss", rename = "negative")]
    /// Unrealized loss expressed as a negative value.
    pub negative_loss: LazyPerBlock<Dollars, Cents>,
}

impl UnrealizedMetrics {
    pub(super) fn new(
        id: CohortId,
        version: Version,
        sources: &Sources,
        price: &LazyPriceWithRatioPerBlock,
        mappings: &Mappings,
    ) -> Self {
        let name = |metric| CohortContext::Utxo.metric_name(id, metric);
        let loss = LazyFiatPerBlock::from_cents_source(
            &name("unrealized_loss"),
            version,
            &sources.unrealized_loss,
            mappings,
        );
        let negative_loss = LazyPerBlock::from_lazy::<NegCentsUnsignedToDollars, Cents>(
            &name("unrealized_loss_neg"),
            version,
            &loss.cents,
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
            nupl: LazyRatioPerBlock::from_lazy_source::<MvrvToNupl, PriceRatio>(
                &name("nupl"),
                version,
                &price.relative.ppm,
            ),
            negative_loss,
        }
    }
}
