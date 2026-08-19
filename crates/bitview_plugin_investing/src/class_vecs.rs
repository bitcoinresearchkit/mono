use bitview_traversable::Traversable;
use brk_types::{Cents, PartsPerMillionSigned64};

use super::{ByDcaClass, dca_stack::DcaStack};
use bitview_compute::{LazyPerBlock, LazyPercentPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct ClassVecs {
    /// Current value and accumulated bitcoin from investing $100 at every UTC
    /// daily close since the named starting year.
    pub dca_stack: ByDcaClass<DcaStack>,
    /// Total dollars invested by the $100-per-day strategy since the named
    /// starting year, divided by the bitcoin it accumulated.
    pub dca_cost_basis: ByDcaClass<Price<LazyPerBlock<Cents>>>,
    /// Current Bitcoin spot price divided by the DCA cost basis accumulated
    /// since the named starting year, minus one.
    pub dca_return: ByDcaClass<LazyPercentPerBlock<PartsPerMillionSigned64>>,
}
