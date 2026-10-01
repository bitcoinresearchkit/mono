use bitview_collections::ByPercentile;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, LazyPercentPerBlock, Price};
use brk_types::{Cents, PartsPerMillion32};

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
    pub min: Price<LazyPerBlock<Cents>>,
    /// Highest creation price among that cohort's unspent outputs.
    pub max: Price<LazyPerBlock<Cents>>,
    /// Creation-price percentiles weighted by that cohort's unspent satoshis.
    pub per_coin: ByPercentile<Price<LazyPerBlock<Cents>>>,
    /// Creation-price percentiles weighted by each output's USD value at
    /// creation.
    pub per_dollar: ByPercentile<Price<LazyPerBlock<Cents>>>,
    /// Share of that cohort's unspent supply with a creation price within 5%
    /// above or below the represented block's spot price.
    pub supply_density: LazyPercentPerBlock<PartsPerMillion32>,
}
