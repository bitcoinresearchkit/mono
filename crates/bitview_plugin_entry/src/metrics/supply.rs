use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_distribution::metrics::SupplyViews;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{LazySpotValuePerBlock, LazyWindowStartVec};
use brk_types::{Cents, Height, Sats, Version};
use vecdb::ReadableBoxedVec;

use super::Sources;

#[derive(Clone, Traversable)]
pub struct SupplyMetrics {
    #[traversable(flatten)]
    views: SupplyViews,
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
        let value = |metric, source| {
            LazySpotValuePerBlock::from_sats_source(
                &name(metric),
                version,
                source,
                mappings,
                prices,
            )
        };
        Self {
            views: SupplyViews::new(
                &name("supply"),
                version,
                &sources.supply,
                all_supply,
                prices,
                mappings,
                windows,
            ),
            in_profit: value("supply_in_profit", &sources.supply_in_profit),
            in_loss: value("supply_in_loss", &sources.supply_in_loss),
        }
    }
}
