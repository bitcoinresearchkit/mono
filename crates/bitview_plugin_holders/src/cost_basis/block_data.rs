use bitview_distribution::state::cost_basis::PercentileResult;
use bitview_primitives::{PERCENTILES_LEN, PartsPerMillion32};
use brk_types::Cents;

#[derive(Clone)]
pub struct CostBasisBlockData {
    pub min: Cents,
    pub max: Cents,
    pub per_coin: [Cents; PERCENTILES_LEN],
    pub per_dollar: [Cents; PERCENTILES_LEN],
    pub supply_density: PartsPerMillion32,
}

impl CostBasisBlockData {
    #[inline(always)]
    pub fn from_percentiles(
        percentiles: PercentileResult,
        supply_density: PartsPerMillion32,
    ) -> Self {
        Self {
            min: percentiles.min_price,
            max: percentiles.max_price,
            per_coin: percentiles.sat_prices,
            per_dollar: percentiles.usd_prices,
            supply_density,
        }
    }
}
