use brk_cohort::UTXOAggregateId;
use brk_traversable::Traversable;
use brk_types::{Cents, PartsPerMillion32, PercentileId};

use crate::internal::{LazyColumnPerBlock, LazyColumnPercentPerBlock, PercentilePrices, Price};

use super::CostBasisSide;

#[derive(Clone, Traversable)]
pub struct CostBasis {
    pub in_profit: CostBasisSide,
    pub in_loss: CostBasisSide,
    pub min: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
    pub max: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
    pub per_coin: PercentilePrices<Price<LazyColumnPerBlock<Cents, PercentileId>>>,
    pub per_dollar: PercentilePrices<Price<LazyColumnPerBlock<Cents, PercentileId>>>,
    pub supply_density: LazyColumnPercentPerBlock<PartsPerMillion32, UTXOAggregateId>,
}
