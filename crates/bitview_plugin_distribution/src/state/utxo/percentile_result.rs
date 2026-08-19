use brk_types::Cents;

use bitview_compute::PERCENTILES_LEN;

#[derive(Default)]
pub struct PercentileResult {
    pub sat_prices: [Cents; PERCENTILES_LEN],
    pub usd_prices: [Cents; PERCENTILES_LEN],
    pub min_price: Cents,
    pub max_price: Cents,
}
