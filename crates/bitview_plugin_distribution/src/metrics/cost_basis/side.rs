use bitview_cohort::UTXOAggregateId;
use bitview_traversable::Traversable;
use brk_types::Cents;

use bitview_compute::{LazyColumnPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct CostBasisSide {
    /// Satoshi-weighted mean creation price of the selected profit or loss side
    /// of the cohort. Returns spot price when that side has no supply.
    pub per_coin: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
    /// Creation-value-weighted mean creation price of the selected profit or
    /// loss side of the cohort. Returns spot price when that side has no
    /// invested value.
    pub per_dollar: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
}
