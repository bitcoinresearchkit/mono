use brk_traversable::Traversable;
use brk_types::{Height, PartsPerMillion32, PartsPerMillionSigned64, Sats, SatsSigned, Version};
use vecdb::{BinaryTransform, CachedBoxedVec, ReadableCloneableVec};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, LazyIndexedVec, LazyPercentPerBlock,
        LazyRollingDeltasAmountFromHeight, LazySpotValuePerBlock, RatioSats, Windows,
    },
};

#[derive(Clone, Traversable)]
pub struct SupplyBase {
    pub total: LazySpotValuePerBlock,
    pub delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
    #[traversable(rename = "dominance")]
    pub dominance: LazyPercentPerBlock<PartsPerMillion32>,
}

impl SupplyBase {
    pub fn from_total(
        cohort_name: &str,
        version: Version,
        total: LazySpotValuePerBlock,
        all_supply: &CachedBoxedVec<Height, Sats>,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let dominance_name = Self::metric_name(cohort_name, "supply_dominance");
        let source = LazyIndexedVec::new(
            &format!("{dominance_name}_ppm_source"),
            version,
            total.sats.height.read_only_boxed_clone(),
            all_supply.clone(),
            |_, supply, all_supply| RatioSats::<PartsPerMillion32>::apply(supply, all_supply),
        );
        let dominance =
            LazyPercentPerBlock::from_height_source(&dominance_name, version, source, indexes);

        Self::new(
            cohort_name,
            version,
            total,
            dominance,
            indexes,
            cached_starts,
        )
    }

    pub fn from_all_total(
        cohort_name: &str,
        version: Version,
        total: LazySpotValuePerBlock,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let dominance_name = Self::metric_name(cohort_name, "supply_dominance");
        let dominance = LazyPercentPerBlock::from_indexed_source(
            &dominance_name,
            version,
            &total.sats.height,
            Self::all_dominance,
            indexes,
        );

        Self::new(
            cohort_name,
            version,
            total,
            dominance,
            indexes,
            cached_starts,
        )
    }

    fn new(
        cohort_name: &str,
        version: Version,
        total: LazySpotValuePerBlock,
        dominance: LazyPercentPerBlock<PartsPerMillion32>,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let delta = LazyRollingDeltasAmountFromHeight::new(
            &Self::metric_name(cohort_name, "supply_delta"),
            version + Version::TWO,
            &total.sats.height,
            cached_starts,
            indexes,
        );

        Self {
            total,
            delta,
            dominance,
        }
    }

    pub fn metric_name(cohort_name: &str, metric: &str) -> String {
        if cohort_name.is_empty() {
            metric.to_owned()
        } else {
            format!("{cohort_name}_{metric}")
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
