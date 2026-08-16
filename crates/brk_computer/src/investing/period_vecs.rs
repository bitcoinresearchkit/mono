use brk_traversable::Traversable;
use brk_types::{Cents, PartsPerMillionSigned64};

use super::{ByDcaCagr, ByDcaPeriod, dca_stack::DcaStack, lump_sum_stack::LumpSumStack};
use crate::internal::{LazyPerBlock, LazyPercentPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct PeriodVecs {
    pub dca_stack: ByDcaPeriod<DcaStack>,
    pub dca_cost_basis: ByDcaPeriod<Price<LazyPerBlock<Cents>>>,
    pub dca_return: ByDcaPeriod<LazyPercentPerBlock<PartsPerMillionSigned64>>,
    pub dca_cagr: ByDcaCagr<LazyPercentPerBlock<PartsPerMillionSigned64>>,
    pub lump_sum_stack: ByDcaPeriod<LumpSumStack>,
    pub lump_sum_return: ByDcaPeriod<LazyPercentPerBlock<PartsPerMillionSigned64>>,
}
