use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::CoinDays;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPerBlockCumulativeRolling, LazyValuePerBlockCumulativeRolling, LazyWindowStartVec,
};
use brk_types::Version;

use super::Sources;

#[derive(Clone, Traversable)]
pub struct ActivityMetrics {
    /// Spent BTC and its value at the spending block's spot price.
    transfer_volume: LazyValuePerBlockCumulativeRolling,
    /// Spent BTC multiplied by its age in days.
    coindays_destroyed: LazyPerBlockCumulativeRolling<CoinDays>,
}

impl ActivityMetrics {
    pub(super) fn new(
        id: CohortId,
        version: Version,
        sources: &Sources,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let name = |metric| CohortContext::Utxo.metric_name(id, metric);
        Self {
            transfer_volume: LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                &name("transfer_volume"),
                version,
                sources.transfer_sats.cumulative_source(),
                sources.transfer_cents.cumulative_source(),
                mappings,
                windows,
            ),
            coindays_destroyed: LazyPerBlockCumulativeRolling::from_cumulative_source(
                &name("coindays_destroyed"),
                version,
                sources.coindays.cumulative_source(),
                windows,
                mappings,
            ),
        }
    }
}
