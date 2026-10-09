use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_distribution::metrics::SupplyBase;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{LazySpotValuePerBlock, LazyWindowStartVec};
use brk_types::{Cents, Height, Sats, Version};
use vecdb::ReadableBoxedVec;

use super::Sources;

#[derive(Clone, Traversable)]
pub struct SupplyMetrics {
    #[traversable(flatten)]
    base: SupplyBase,
    /// Unspent supply whose creation price is at or below current spot price.
    in_profit: LazySpotValuePerBlock,
    /// Unspent supply whose creation price is above current spot price.
    in_loss: LazySpotValuePerBlock,
}

impl SupplyMetrics {
    pub(super) fn new(
        id: CohortId,
        version: Version,
        sources: &Sources,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        prices: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Self {
        let name = |metric| CohortContext::Utxo.metric_name(id, metric);
        let total = LazySpotValuePerBlock::from_sats_source(
            &name("supply"),
            version,
            &sources.supply,
            mappings,
            prices,
        );
        Self {
            base: SupplyBase::new(
                CohortContext::Utxo,
                id,
                version,
                total,
                all_supply,
                mappings,
                windows,
            ),
            in_profit: LazySpotValuePerBlock::from_sats_source(
                &name("supply_in_profit"),
                version,
                &sources.supply_in_profit,
                mappings,
                prices,
            ),
            in_loss: LazySpotValuePerBlock::from_sats_source(
                &name("supply_in_loss"),
                version,
                &sources.supply_in_loss,
                mappings,
                prices,
            ),
        }
    }
}
