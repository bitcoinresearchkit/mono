use bitview_traversable::Traversable;
use brk_types::{Cents, PartsPerMillionSigned64};

use super::{ByDcaClass, dca_stack::DcaStack};
use bitview_compute::{LazyPerBlock, LazyPercentPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct ClassVecs {
    /// Value at the represented block and accumulated bitcoin from investing
    /// $100 at every UTC daily close since a starting year.
    pub dca_stack: ByDcaClass<DcaStack>,
    /// Total dollars invested by the $100-per-day strategy since a starting
    /// year, divided by the bitcoin it accumulated.
    pub dca_cost_basis: ByDcaClass<Price<LazyPerBlock<Cents>>>,
    /// Bitcoin spot price at the represented block divided by the
    /// dollar-cost-averaging cost basis accumulated since a starting year,
    /// minus one. Positive values mean the strategy is in profit and negative
    /// values mean it is in loss.
    pub dca_return: ByDcaClass<LazyPercentPerBlock<PartsPerMillionSigned64>>,
}
