use bitview_primitives::{BoundedRatio, Day1, Percent, Ratio};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlock, Price};
use brk_types::Cents;
use vecdb::{AnyStoredVec, PcoVec, Rw, StorageMode, WritableVec};

use super::{EntryVecs, ModeResult, Percentiles, PriceBands, ProfitPercentiles};

#[derive(Traversable)]
pub struct ModeVecs<M: StorageMode = Rw> {
    /// The low tail of the mode's prior finite supply-in-profit shares, as the
    /// supply-in-loss thresholds are the high tail of its loss shares: each is the complement of
    /// the loss threshold at the matching rank (95th is 5th), over the same history (the
    /// represented block excluded, unavailable before 52,560 prior block observations).
    #[traversable(lead = "Thin-profit threshold: the share of coins in profit it calls extreme.")]
    pub supply_in_profit_share_threshold: ProfitPercentiles<LazyPerBlock<Percent, BoundedRatio>>,
    /// Historical supply-in-loss share that Bedrock treats as a stressed
    /// condition for this mode. It is a nearest-rank percentile of the
    /// mode's prior finite block loss shares. The represented block is excluded,
    /// and the value is unavailable until its loss share exists and at least
    /// 52,560 prior block observations are available. Stored as a bounded share and
    /// exposed as a percentage. Calibration remains full precision.
    #[traversable(lead = "Stress threshold: the share of coins in loss it calls extreme.")]
    pub supply_in_loss_share_threshold: Percentiles<LazyPerBlock<Percent, BoundedRatio>>,
    /// The part of the mode's prior finite block shares of supply in profit below the day's
    /// close (its last block, UTC), over the history the thresholds use: at most 0.01 when the
    /// close's supply-in-loss share reaches the 99th-percentile threshold, and under 0.01 only
    /// then; so for the others. Null where the thresholds are.
    #[traversable(lead = "Profit rank: the part of earlier blocks with less of the supply in profit than today.")]
    pub supply_in_profit_share_rank: M::Stored<PcoVec<Day1, Ratio>>,
    #[traversable(flatten)]
    /// Bedrock maps historically stressed supply-in-loss shares onto the
    /// represented block's mode-weighted distribution of UTXO creation prices to
    /// estimate lower price bands. A UTXO's creation price is Bitcoin's spot
    /// price when that output was created.
    pub prices: PriceBands<Price<LazyPerBlock<Cents>>>,
    #[traversable(flatten)]
    pub floor: EntryVecs<M>,
    #[traversable(hidden)]
    pub(crate) supply_in_loss_share_threshold_stored:
        Percentiles<CachedSeries<BoundedRatio, M>>,
    #[traversable(hidden)]
    pub(crate) prices_stored: PriceBands<CachedSeries<Cents, M>>,
}

impl ModeVecs {
    /// The vecs the walk stores: the thresholds and the prices.
    pub(crate) fn walk_stored_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.supply_in_loss_share_threshold_stored
            .iter_any_stored_mut()
            .chain(self.prices_stored.iter_any_stored_mut())
    }

    pub(crate) fn push(&mut self, result: &ModeResult) {
        for (vec, &value) in self
            .supply_in_loss_share_threshold_stored
            .iter_mut()
            .zip(result.supply_in_loss_threshold.iter())
        {
            vec.push(value);
        }
        for (vec, &value) in self.prices_stored.iter_mut().zip(result.prices.iter()) {
            vec.push(value);
        }
    }
}
