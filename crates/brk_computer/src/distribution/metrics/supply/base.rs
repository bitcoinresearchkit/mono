use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{Height, PartsPerMillion32, PartsPerMillionSigned64, Sats, SatsSigned, Version};
use vecdb::{
    AnyStoredVec, AnyVec, BinaryTransform, EagerVec, Exit, ImportableVec, PcoVec, ReadOnlyClone,
    ReadableCloneableVec, Rw, StorageMode, WritableVec,
};

use crate::distribution::state::{CohortState, CostBasisOps, RealizedOps};

use crate::internal::{
    LazyIndexedVec, LazyPercentPerBlock, LazyRollingDeltasAmountFromHeight, LazySpotValuePerBlock,
    RatioSats,
};

use crate::distribution::metrics::{AllSupplyCache, ImportConfig};

/// Base supply metrics: total supply + dominance (share of circulating).
#[derive(Traversable)]
pub struct SupplyBase<M: StorageMode = Rw> {
    pub total: LazySpotValuePerBlock,
    pub delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
    #[traversable(rename = "dominance")]
    pub dominance: LazyPercentPerBlock<PartsPerMillion32>,
    #[traversable(hidden)]
    source: Option<M::Stored<EagerVec<PcoVec<Height, Sats>>>>,
}

impl SupplyBase {
    pub(crate) fn forced_import(cfg: &ImportConfig, all_supply: &AllSupplyCache) -> Result<Self> {
        let name = cfg.name("supply");
        let sats_source =
            EagerVec::forced_import(cfg.db, &format!("{name}_sats"), cfg.version)?;
        let supply = LazySpotValuePerBlock::from_sats_source(
            &name,
            cfg.version,
            sats_source.read_only_clone(),
            cfg.indexes,
            cfg.spot_price,
        );
        let name = cfg.name("supply_dominance");
        let dominance_source = LazyIndexedVec::new(
            &format!("{name}_ppm_source"),
            cfg.version,
            supply.sats.height.read_only_boxed_clone(),
            all_supply.cached_boxed_clone(),
            |_, supply, all_supply| RatioSats::<PartsPerMillion32>::apply(supply, all_supply),
        );
        let dominance = LazyPercentPerBlock::from_height_source(
            &name,
            cfg.version,
            dominance_source,
            cfg.indexes,
        );

        Ok(Self::new(cfg, supply, dominance, Some(sats_source)))
    }

    pub(crate) fn from_lazy(
        cfg: &ImportConfig,
        supply: LazySpotValuePerBlock,
        all_supply: &AllSupplyCache,
    ) -> Self {
        let name = cfg.name("supply_dominance");
        let source = LazyIndexedVec::new(
            &format!("{name}_ppm_source"),
            cfg.version,
            supply.sats.height.read_only_boxed_clone(),
            all_supply.cached_boxed_clone(),
            |_, supply, all_supply| RatioSats::<PartsPerMillion32>::apply(supply, all_supply),
        );
        let dominance =
            LazyPercentPerBlock::from_height_source(&name, cfg.version, source, cfg.indexes);

        Self::new(cfg, supply, dominance, None)
    }

    pub(crate) fn from_lazy_all(cfg: &ImportConfig, supply: LazySpotValuePerBlock) -> Self {
        let dominance = LazyPercentPerBlock::from_indexed_source(
            &cfg.name("supply_dominance"),
            cfg.version,
            &supply.sats.height,
            Self::all_dominance,
            cfg.indexes,
        );

        Self::new(cfg, supply, dominance, None)
    }

    fn all_dominance(_height: Height, supply: Sats) -> PartsPerMillion32 {
        RatioSats::<PartsPerMillion32>::apply(supply, supply)
    }

    fn new(
        cfg: &ImportConfig,
        supply: LazySpotValuePerBlock,
        dominance: LazyPercentPerBlock<PartsPerMillion32>,
        source: Option<EagerVec<PcoVec<Height, Sats>>>,
    ) -> Self {
        let delta = LazyRollingDeltasAmountFromHeight::new(
            &cfg.name("supply_delta"),
            cfg.version + Version::TWO,
            &supply.sats.height,
            cfg.cached_starts,
            cfg.indexes,
        );

        Self {
            total: supply,
            delta,
            dominance,
            source,
        }
    }

    pub(crate) fn min_len(&self) -> usize {
        self.total.sats.height.len()
    }

    #[inline(always)]
    pub(crate) fn push_state(&mut self, state: &CohortState<impl RealizedOps, impl CostBasisOps>) {
        if let Some(source) = self.source.as_mut() {
            source.push(state.supply.value);
        }
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.source
            .iter_mut()
            .map(|source| source as &mut dyn AnyStoredVec)
            .collect()
    }

    pub(crate) fn compute_from_stateful(
        &mut self,
        starting_lengths: &Lengths,
        others: &[&Self],
        exit: &Exit,
    ) -> Result<()> {
        if let Some(source) = self.source.as_mut() {
            source.compute_sum_of_others(
                starting_lengths.height,
                &others
                    .iter()
                    .map(|v| &v.total.sats.height)
                    .collect::<Vec<_>>(),
                exit,
            )?;
        }
        Ok(())
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
