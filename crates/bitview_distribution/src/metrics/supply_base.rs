use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned64};
use bitview_transforms::Quotient;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFixedRatioPerBlock, LazyIndexedVec, LazyRollingDeltasAmountFromHeight,
    LazySpotValuePerBlock, LazyWindowStartVec,
};
use brk_types::{Height, Sats, SatsSigned, Version};
use vecdb::{BinaryTransform, ReadableCloneableVec};

#[derive(Clone, Traversable)]
pub struct SupplyBase {
    total: LazySpotValuePerBlock,
    pub delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
    #[traversable(rename = "dominance")]
    pub dominance: LazyFixedRatioPerBlock<PartsPerMillion32>,
}

impl SupplyBase {
    pub fn new(
        context: CohortContext,
        cohort: CohortId,
        version: Version,
        total: LazySpotValuePerBlock,
        all_supply: &impl ReadableCloneableVec<Height, Sats>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let dominance_name = context.metric_name(cohort, "supply_dominance");
        let source = LazyIndexedVec::new(
            &format!("{dominance_name}_ppm_source"),
            version,
            &total.sats.height,
            all_supply,
            |_, supply, all_supply| Quotient::<PartsPerMillion32>::apply(supply, all_supply),
        );
        let dominance =
            LazyFixedRatioPerBlock::from_height_source(&dominance_name, version, &source, mappings);

        Self::from_parts(
            context,
            cohort,
            version,
            total,
            dominance,
            mappings,
            window_starts,
        )
    }

    fn from_parts(
        context: CohortContext,
        cohort: CohortId,
        version: Version,
        total: LazySpotValuePerBlock,
        dominance: LazyFixedRatioPerBlock<PartsPerMillion32>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let delta = LazyRollingDeltasAmountFromHeight::new(
            &context.metric_name(cohort, "supply_delta"),
            version + Version::TWO,
            &total.sats.height,
            window_starts,
            mappings,
        );

        Self {
            total,
            delta,
            dominance,
        }
    }
}
