use bitview_cohort::UTXOAggregateId;
use bitview_traversable::Traversable;
use brk_types::{Cents, PartsPerMillion32, PercentileId};

use bitview_compute::{LazyColumnPerBlock, LazyColumnPercentPerBlock, PercentilePrices, Price};

use super::CostBasisSide;

#[derive(Clone, Traversable)]
pub struct CostBasis {
    /// Restricts that cohort to outputs whose creation price is less than or
    /// equal to the represented block's spot price.
    pub in_profit: CostBasisSide,
    /// Restricts that cohort to outputs whose creation price is greater than the
    /// represented block's spot price.
    pub in_loss: CostBasisSide,
    /// Lowest creation price among that cohort's unspent outputs.
    pub min: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
    /// Highest creation price among that cohort's unspent outputs.
    pub max: Price<LazyColumnPerBlock<Cents, UTXOAggregateId>>,
    /// Creation-price percentiles weighted by that cohort's unspent satoshis.
    pub per_coin: PercentilePrices<Price<LazyColumnPerBlock<Cents, PercentileId>>>,
    /// Creation-price percentiles weighted by each output's USD value at
    /// creation.
    pub per_dollar: PercentilePrices<Price<LazyColumnPerBlock<Cents, PercentileId>>>,
    /// Share of that cohort's unspent supply with a creation price within 5%
    /// above or below the represented block's spot price.
    pub supply_density: LazyColumnPercentPerBlock<PartsPerMillion32, UTXOAggregateId>,
}
