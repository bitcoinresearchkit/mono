use bitview_traversable::Traversable;
use brk_types::Cents;

use bitview_compute::ByLookbackPeriod;
use bitview_compute::{LazyPerBlock, Price};
#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Bitcoin spot price at the first block in the named trailing
    /// monotonic-time window.
    #[traversable(flatten)]
    pub price_past: ByLookbackPeriod<Price<LazyPerBlock<Cents>>>,
}
