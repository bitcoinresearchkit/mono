use bitview_traversable::Traversable;
use brk_types::Cents;

use bitview_vecs::{LazyPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct CostBasisSide {
    /// Within that subset, the satoshi-weighted mean creation price. Returns the
    /// represented block's spot price when the subset has no supply.
    pub per_coin: Price<LazyPerBlock<Cents>>,
    /// Within that subset, the mean creation price weighted by each output's USD
    /// value at creation. Returns the represented block's spot price when the
    /// subset has no invested value.
    pub per_dollar: Price<LazyPerBlock<Cents>>,
}
