use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, AnyVec, EagerVec, Exit, ImportableVec, PcoVec, ReadOnlyClone, Rw, StorageMode,
    WritableVec,
};

use crate::distribution::state::UnrealizedState;

use crate::internal::{
    HalveCents, HalveDollars, HalveSats, HalveSatsToBitcoin, LazySpotValuePerBlock,
    LazyValuePerBlock,
};

use crate::distribution::metrics::{AllSupplyCache, ImportConfig};

use super::SupplyBase;

/// Core supply metrics: total + halved + in_profit/in_loss.
#[derive(Deref, DerefMut, Traversable)]
pub struct SupplyCore<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub base: SupplyBase<M>,

    pub half: LazyValuePerBlock,
    pub in_profit: LazySpotValuePerBlock,
    pub in_loss: LazySpotValuePerBlock,
    #[traversable(hidden)]
    in_profit_source: Option<M::Stored<EagerVec<PcoVec<Height, Sats>>>>,
    #[traversable(hidden)]
    in_loss_source: Option<M::Stored<EagerVec<PcoVec<Height, Sats>>>>,
}

impl SupplyCore {
    pub(crate) fn forced_import(cfg: &ImportConfig, all_supply: &AllSupplyCache) -> Result<Self> {
        let base = SupplyBase::forced_import(cfg, all_supply)?;
        let (in_profit, in_profit_source) = Self::import_spot(cfg, "supply_in_profit")?;
        let (in_loss, in_loss_source) = Self::import_spot(cfg, "supply_in_loss")?;
        Ok(Self::new(
            cfg,
            base,
            in_profit,
            in_loss,
            Some(in_profit_source),
            Some(in_loss_source),
        ))
    }

    pub(crate) fn from_lazy(
        cfg: &ImportConfig,
        total: LazySpotValuePerBlock,
        in_profit: LazySpotValuePerBlock,
        in_loss: LazySpotValuePerBlock,
        all_supply: &AllSupplyCache,
    ) -> Self {
        Self::new(
            cfg,
            SupplyBase::from_lazy(cfg, total, all_supply),
            in_profit,
            in_loss,
            None,
            None,
        )
    }

    pub(crate) fn from_lazy_all(
        cfg: &ImportConfig,
        total: LazySpotValuePerBlock,
        in_profit: LazySpotValuePerBlock,
        in_loss: LazySpotValuePerBlock,
    ) -> Self {
        Self::new(
            cfg,
            SupplyBase::from_lazy_all(cfg, total),
            in_profit,
            in_loss,
            None,
            None,
        )
    }

    fn import_spot(
        cfg: &ImportConfig,
        suffix: &str,
    ) -> Result<(LazySpotValuePerBlock, EagerVec<PcoVec<Height, Sats>>)> {
        let name = cfg.name(suffix);
        let source = EagerVec::forced_import(cfg.db, &format!("{name}_sats"), cfg.version)?;
        let series = LazySpotValuePerBlock::from_sats_source(
            &name,
            cfg.version,
            source.read_only_clone(),
            cfg.indexes,
            cfg.spot_price,
        );
        Ok((series, source))
    }

    fn new(
        cfg: &ImportConfig,
        base: SupplyBase,
        in_profit: LazySpotValuePerBlock,
        in_loss: LazySpotValuePerBlock,
        in_profit_source: Option<EagerVec<PcoVec<Height, Sats>>>,
        in_loss_source: Option<EagerVec<PcoVec<Height, Sats>>>,
    ) -> Self {
        let half = LazyValuePerBlock::from_spot_block_source::<
            HalveSats,
            HalveSatsToBitcoin,
            HalveCents,
            HalveDollars,
        >(&cfg.name("supply_half"), &base.total, cfg.version);

        Self {
            base,
            half,
            in_profit,
            in_loss,
            in_profit_source,
            in_loss_source,
        }
    }

    pub(crate) fn min_len(&self) -> usize {
        self.base
            .min_len()
            .min(self.in_profit.sats.height.len())
            .min(self.in_loss.sats.height.len())
    }

    #[inline(always)]
    pub(crate) fn push_profitability(&mut self, state: &UnrealizedState) {
        if let Some(source) = self.in_profit_source.as_mut() {
            source.push(state.supply_in_profit);
        }
        if let Some(source) = self.in_loss_source.as_mut() {
            source.push(state.supply_in_loss);
        }
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.base.collect_vecs_mut();
        vecs.extend(
            self.in_profit_source
                .iter_mut()
                .map(|source| source as &mut dyn AnyStoredVec),
        );
        vecs.extend(
            self.in_loss_source
                .iter_mut()
                .map(|source| source as &mut dyn AnyStoredVec),
        );
        vecs
    }

    pub(crate) fn validate_computed_versions(&mut self, _base_version: Version) -> Result<()> {
        Ok(())
    }

    pub(crate) fn compute_from_stateful(
        &mut self,
        starting_lengths: &Lengths,
        others: &[&Self],
        exit: &Exit,
    ) -> Result<()> {
        let base_refs: Vec<&SupplyBase> = others.iter().map(|o| &o.base).collect();
        self.base
            .compute_from_stateful(starting_lengths, &base_refs, exit)?;
        if let Some(source) = self.in_profit_source.as_mut() {
            source.compute_sum_of_others(
                starting_lengths.height,
                &others
                    .iter()
                    .map(|other| &other.in_profit.sats.height)
                    .collect::<Vec<_>>(),
                exit,
            )?;
        }
        if let Some(source) = self.in_loss_source.as_mut() {
            source.compute_sum_of_others(
                starting_lengths.height,
                &others
                    .iter()
                    .map(|other| &other.in_loss.sats.height)
                    .collect::<Vec<_>>(),
                exit,
            )?;
        }
        Ok(())
    }
}
