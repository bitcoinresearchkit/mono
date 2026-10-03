use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{StoredF64, StoredU64};
use bitview_transforms::{DaysToYears, StoredU64ToCents, StoredU64ToSats};
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyPerBlock, LazyPerBlockCumulativeRolling, LazyValuePerBlockCumulativeRolling,
    LazyWindowStartVec,
};
use brk_types::{Height, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

use crate::columns::Columns;

#[derive(Clone, Traversable)]
pub struct Activity {
    pub transfer_volume: LazyValuePerBlockCumulativeRolling,
    pub transfer_volume_in_profit: LazyValuePerBlockCumulativeRolling,
    pub transfer_volume_in_loss: LazyValuePerBlockCumulativeRolling,
    /// Coin days destroyed (CDD): spent coin amounts multiplied by their age in days.
    pub coindays_destroyed: LazyPerBlockCumulativeRolling<StoredF64>,
    pub coinyears_destroyed: LazyPerBlock<StoredF64, StoredF64>,
}
impl Activity {
    pub(crate) fn new(
        id: AgeAggregateId,
        v: Version,
        c: &Columns,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let value = |metric: &str,
                     sats: &CachedSeries<Height, StoredU64>,
                     cents: &CachedSeries<Height, StoredU64>| {
            let name = id.metric_name(metric);
            let sats = LazyVec::transformed::<StoredU64ToSats>(
                &format!("{name}_cumulative_sats"),
                v,
                sats.read_only_boxed_clone(),
            );
            let cents = LazyVec::transformed::<StoredU64ToCents>(
                &format!("{name}_cumulative_cents"),
                v,
                cents.read_only_boxed_clone(),
            );
            LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                &name, v, &sats, &cents, mappings, windows,
            )
        };
        let transfer_volume = value("transfer_volume", &c.volume_sats, &c.volume_cents);
        let transfer_volume_in_profit = value(
            "transfer_volume_in_profit",
            &c.volume_profit_sats,
            &c.volume_profit_cents,
        );
        let transfer_volume_in_loss = value(
            "transfer_volume_in_loss",
            &c.volume_loss_sats,
            &c.volume_loss_cents,
        );
        let coindays_destroyed = LazyPerBlockCumulativeRolling::from_cumulative_source(
            &id.metric_name("coindays_destroyed"),
            v,
            &c.cdd,
            windows,
            mappings,
        );
        let coinyears_destroyed = LazyPerBlock::from_height_source::<DaysToYears>(
            &id.metric_name("coinyears_destroyed"),
            v,
            &coindays_destroyed.sum._1y.height,
            mappings,
        );
        Self {
            transfer_volume,
            transfer_volume_in_profit,
            transfer_volume_in_loss,
            coindays_destroyed,
            coinyears_destroyed,
        }
    }
}
