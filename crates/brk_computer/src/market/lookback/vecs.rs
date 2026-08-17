use brk_traversable::Traversable;
use brk_types::Cents;

use super::ByLookbackPeriod;
use crate::internal::{LazyPerBlock, Price};
#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Bitcoin spot price at the first block in the named trailing
    /// monotonic-time window.
    #[traversable(flatten)]
    pub price_past: ByLookbackPeriod<Price<LazyPerBlock<Cents>>>,
}
