use bitview_cohort::UTXOAggregateId;
use bitview_traversable::Traversable;
use brk_types::Cents;

use bitview_compute::{LazyColumnPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct CostBasisSide {
    /// Within that subset, the satoshi-weighted mean creation price. Returns the
    /// represented block's spot price when the subset has no supply.
    pub per_coin: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
    /// Within that subset, the mean creation price weighted by each output's USD
    /// value at creation. Returns the represented block's spot price when the
    /// subset has no invested value.
    pub per_dollar: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
}
