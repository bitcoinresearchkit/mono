use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::RatioSats;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyIndexedVec, LazyPercentPerBlock, LazyRollingDeltasAmountFromHeight, LazySpotValuePerBlock,
    LazyWindowStartVec,
};
use brk_types::{Height, PartsPerMillion32, PartsPerMillionSigned64, Sats, SatsSigned, Version};
use vecdb::{BinaryTransform, LazyVec, ReadableCloneableVec};

#[derive(Clone, Traversable)]
pub struct SupplyBase {
    pub total: LazySpotValuePerBlock,
    pub delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
    #[traversable(rename = "dominance")]
    pub dominance: LazyPercentPerBlock<PartsPerMillion32>,
}

impl SupplyBase {
    pub fn from_total(
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
            |_, supply, all_supply| RatioSats::<PartsPerMillion32>::apply(supply, all_supply),
        );
        let dominance =
            LazyPercentPerBlock::from_height_source(&dominance_name, version, &source, mappings);

        Self::new(
            context,
            cohort,
            version,
            total,
            dominance,
            mappings,
            window_starts,
        )
    }

    pub fn from_all_total(
        version: Version,
        total: LazySpotValuePerBlock,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let dominance_name = CohortContext::Utxo.metric_name(CohortId::All, "supply_dominance");
        let source = LazyVec::init(
            &format!("{dominance_name}_ppm_source"),
            version,
            total.sats.height.read_only_boxed_clone(),
            Self::all_dominance,
        );
        let dominance =
            LazyPercentPerBlock::from_height_source(&dominance_name, version, &source, mappings);

        Self::new(
            CohortContext::Utxo,
            CohortId::All,
            version,
            total,
            dominance,
            mappings,
            window_starts,
        )
    }

    fn new(
        context: CohortContext,
        cohort: CohortId,
        version: Version,
        total: LazySpotValuePerBlock,
        dominance: LazyPercentPerBlock<PartsPerMillion32>,
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

    fn all_dominance(_height: Height, supply: Sats) -> PartsPerMillion32 {
        RatioSats::<PartsPerMillion32>::apply(supply, supply)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_dominance_preserves_zero_supply() {
        assert_eq!(
            SupplyBase::all_dominance(Height::ZERO, Sats::ZERO),
            PartsPerMillion32::ZERO
        );
        assert_eq!(
            SupplyBase::all_dominance(Height::ZERO, Sats::new(1)),
            PartsPerMillion32::ONE
        );
    }
}
