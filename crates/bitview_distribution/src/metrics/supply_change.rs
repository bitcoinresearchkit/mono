use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned64};
use bitview_transforms::Quotient;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyIndexedVec, LazyPercentPerBlock, LazyRollingDeltasAmountFromHeight, LazySpotValuePerBlock,
    LazyWindowStartVec,
};
use brk_types::{Height, Sats, SatsSigned, Version};
use vecdb::{BinaryTransform, ReadableCloneableVec};

/// A cohort's supply change over each trailing window and its share of all supply.
#[derive(Clone, Traversable)]
pub struct SupplyChange {
    /// Change in the cohort's supply over a trailing window, with the relative
    /// change measured against the window's starting value.
    delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
    /// Share of all unspent supply held by the cohort.
    share: LazyPercentPerBlock<PartsPerMillion32>,
}

impl SupplyChange {
    pub fn new(
        context: CohortContext,
        cohort: CohortId,
        version: Version,
        total: &LazySpotValuePerBlock,
        all_supply: &impl ReadableCloneableVec<Height, Sats>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let share_name = context.metric_name(cohort, "supply_share");
        let source = LazyIndexedVec::new(
            &format!("{share_name}_ppm_source"),
            version,
            &total.sats.height,
            all_supply,
            |_, supply, all_supply| Quotient::<PartsPerMillion32>::apply(supply, all_supply),
        );
        let share =
            LazyPercentPerBlock::from_height_source(&share_name, version, &source, mappings);
        let delta = LazyRollingDeltasAmountFromHeight::new(
            &context.metric_name(cohort, "supply_delta"),
            version + Version::TWO,
            &total.sats.height,
            window_starts,
            mappings,
        );
        Self { delta, share }
    }
}
