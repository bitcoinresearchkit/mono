use brk_traversable::Traversable;
use brk_types::{Cents, PartsPerMillionSigned64};

use super::{ByDcaClass, dca_stack::DcaStack};
use crate::internal::{LazyPerBlock, LazyPercentPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct ClassVecs {
    pub dca_stack: ByDcaClass<DcaStack>,
    pub dca_cost_basis: ByDcaClass<Price<LazyPerBlock<Cents>>>,
    pub dca_return: ByDcaClass<LazyPercentPerBlock<PartsPerMillionSigned64>>,
}
