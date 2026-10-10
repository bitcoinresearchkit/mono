use bitview_collections::ByPercentile;
use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{Density, LazyPerBlock, LazyPercentPerBlock, Price};
use brk_types::Cents;

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
    /// Share of that cohort's unspent supply with a creation price near spot.
    pub supply_density: Density<LazyPercentPerBlock<PartsPerMillion32>>,
    /// Share of that cohort's invested capital (satoshis times creation price)
    /// with a creation price near spot.
    pub capital_density: Density<LazyPercentPerBlock<PartsPerMillion32>>,
}
