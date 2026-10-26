use bitview_compute::{Retained, SortedWindow};
use bitview_primitives::{DollarDaysPerCoin, Float, Float64};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, PerBlock, PricedPerBlockCumulativeRolling};
use brk_types::Height;
use vecdb::{PcoVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Reserve Risk: spot price in USD divided by the HODL bank. Lower positive
    /// values mean spot is cheaper relative to the accumulated holder reserve;
    /// higher values mean it is more expensive. Undefined (null) when the HODL
    /// bank is zero.
    #[traversable(lead = "Reserve Risk: the price against long-term holders' conviction, lowest when they're confident and the price is low.")]
    pub value: LazyPerBlock<Float, Float64>,
    /// HODL bank: running sum of spot price in USD minus the 30-day VOCDD
    /// median, each block weighted by the time since the previous block in days
    /// (on monotonic timestamps). It represents the model's accumulated holder
    /// reserve and is the denominator of Reserve Risk.
    #[traversable(lead = "HODL bank: what long-term holders have given up, in all, by not selling.")]
    pub hodl_bank: LazyPerBlock<Float, Float64>,
    /// The HODL bank's running sum at double precision, continued block after block.
    #[traversable(hidden)]
    pub(crate) hodl_bank_sum: PerBlock<Float64, M>,
    /// Supply-adjusted value of coin days destroyed (VOCDD): spot price in USD
    /// multiplied by the block's coin days destroyed and divided by circulating
    /// supply in BTC. Reported in USD-days per BTC of circulating supply. Returns
    /// zero when circulating supply is zero.
    #[traversable(lead = "VOCDD: coin days destroyed, at the price of the day, per bitcoin in circulation.")]
    pub vocdd: PricedPerBlockCumulativeRolling<DollarDaysPerCoin, M>,
    /// Median over the blocks of the trailing 30 days of `vocdd.sum._24h`, in
    /// USD-days per BTC: the HODL bank's input. None until that sum has a value
    /// (a day after the first exchange price), its windows starting there.
    #[traversable(hidden)]
    pub(crate) vocdd_median_1m: M::Stored<PcoVec<Height, DollarDaysPerCoin>>,
    pub(crate) vocdd_median_1m_window: M::WriteOnly<Retained<SortedWindow>>,
}
