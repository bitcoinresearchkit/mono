use bitview_traversable::Traversable;
use brk_types::{Cents, PartsPerMillionSigned64};

use super::{dca_stack::DcaStack, lump_sum_stack::LumpSumStack};
use bitview_compute::{ByDcaCagr, ByDcaPeriod, LazyPerBlock, LazyPercentPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct PeriodVecs {
    /// Value at the represented block and accumulated bitcoin from investing
    /// $100 at every UTC daily close during a trailing period.
    pub dca_stack: ByDcaPeriod<DcaStack>,
    /// Total dollars invested by the trailing-period $100-per-day strategy,
    /// divided by the bitcoin it accumulated.
    pub dca_cost_basis: ByDcaPeriod<Price<LazyPerBlock<Cents>>>,
    /// Bitcoin spot price at the represented block divided by the trailing
    /// dollar-cost-averaging (DCA) cost basis, minus one. Positive values mean
    /// the strategy is in profit and negative values mean it is in loss.
    pub dca_return: ByDcaPeriod<LazyPercentPerBlock<PartsPerMillionSigned64>>,
    /// Annualized trailing-period DCA return over a whole-year period: `(1 +
    /// return)^(1 / years) - 1`; this is not a cash-flow internal rate of
    /// return. Positive values are annualized gains and negative values are
    /// annualized losses.
    pub dca_cagr: ByDcaCagr<LazyPercentPerBlock<PartsPerMillionSigned64>>,
    /// Value at the represented block and bitcoin from investing $100 times a
    /// trailing period's day count at that period's first block-level spot
    /// price.
    pub lump_sum_stack: ByDcaPeriod<LumpSumStack>,
    /// Bitcoin spot price at the represented block divided by its value at the
    /// first block in a trailing period, minus one. Positive values mean the
    /// lump-sum investment gained value and negative values mean it lost value.
    pub lump_sum_return: ByDcaPeriod<LazyPercentPerBlock<PartsPerMillionSigned64>>,
}
