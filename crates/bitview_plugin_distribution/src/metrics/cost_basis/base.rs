use bitview_traversable::Traversable;
use brk_cohort::UTXOAggregateId;
use brk_types::{Cents, PartsPerMillion32, PercentileId};

use bitview_compute::{LazyColumnPerBlock, LazyColumnPercentPerBlock, PercentilePrices, Price};

use super::CostBasisSide;

#[derive(Clone, Traversable)]
pub struct CostBasis {
    pub in_profit: CostBasisSide,
    pub in_loss: CostBasisSide,
    /// Lowest creation price represented by an unspent output in the selected
    /// cohort.
    pub min: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
    /// Highest creation price represented by an unspent output in the selected
    /// cohort.
    pub max: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
    /// Creation-price percentiles weighted by unspent satoshis in the selected
    /// cohort.
    pub per_coin: PercentilePrices<Price<LazyColumnPerBlock<Cents, PercentileId>>>,
    /// Creation-price percentiles weighted by creation-date USD value in the
    /// selected cohort.
    pub per_dollar: PercentilePrices<Price<LazyColumnPerBlock<Cents, PercentileId>>>,
    /// Share of the selected cohort's unspent supply with a creation price
    /// within 5% above or below spot price.
    pub supply_density: LazyColumnPercentPerBlock<PartsPerMillion32, UTXOAggregateId>,
}
