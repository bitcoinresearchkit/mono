use brk_cohort::UTXOAggregateId;
use brk_traversable::Traversable;
use brk_types::Cents;

use crate::internal::{LazyColumnPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct CostBasisSide {
    pub per_coin: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
    pub per_dollar: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
}
