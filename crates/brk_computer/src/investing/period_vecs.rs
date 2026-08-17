use brk_traversable::Traversable;
use brk_types::{Cents, PartsPerMillionSigned64};

use super::{ByDcaCagr, ByDcaPeriod, dca_stack::DcaStack, lump_sum_stack::LumpSumStack};
use crate::internal::{LazyPerBlock, LazyPercentPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct PeriodVecs {
    /// Current value and accumulated bitcoin from investing $100 at every UTC
    /// daily close during the named trailing period.
    pub dca_stack: ByDcaPeriod<DcaStack>,
    /// Total dollars invested by the trailing-period $100-per-day strategy,
    /// divided by the bitcoin it accumulated.
    pub dca_cost_basis: ByDcaPeriod<Price<LazyPerBlock<Cents>>>,
    /// Current Bitcoin spot price divided by the trailing-period DCA cost
    /// basis, minus one.
    pub dca_return: ByDcaPeriod<LazyPercentPerBlock<PartsPerMillionSigned64>>,
    /// Annualized trailing-period DCA return over the named whole-year period:
    /// `(1 + return)^(1 / years) - 1`; this is not a cash-flow IRR.
    pub dca_cagr: ByDcaCagr<LazyPercentPerBlock<PartsPerMillionSigned64>>,
    /// Current value and bitcoin from investing $100 times the named period's
    /// day count at the period's first block-level spot price.
    pub lump_sum_stack: ByDcaPeriod<LumpSumStack>,
    /// Current Bitcoin spot price divided by its value at the first block in
    /// the named trailing period, minus one.
    pub lump_sum_return: ByDcaPeriod<LazyPercentPerBlock<PartsPerMillionSigned64>>,
}
