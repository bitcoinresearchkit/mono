use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_distribution_common::metrics::SupplyBase;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_transforms::{HalveDollars, HalveSatsToBitcoin};
use bitview_traversable::Traversable;
use bitview_vecs::{LazySpotValuePerBlock, LazyValuePerBlock, LazyWindowStartVec};
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Halve, ReadableBoxedVec};

use super::Sources;

#[derive(Clone, Traversable)]
pub struct SupplyMetrics {
    #[traversable(flatten)]
    pub base: SupplyBase,
    /// Unspent supply whose creation price is at or below current spot price.
    pub in_profit: LazySpotValuePerBlock,
    /// Unspent supply whose creation price is above current spot price.
    pub in_loss: LazySpotValuePerBlock,
    /// Half of this cohort's unspent supply.
    pub half: LazyValuePerBlock,
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
        let half = LazyValuePerBlock::from_spot_block_source::<
            Halve,
            HalveSatsToBitcoin,
            Halve,
            HalveDollars,
        >(&name("supply_half"), &total, version);
        Self {
            base: SupplyBase::from_total(
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
            half,
        }
    }
}
