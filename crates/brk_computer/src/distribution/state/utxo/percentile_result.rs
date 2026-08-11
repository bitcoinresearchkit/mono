use brk_types::Cents;

use crate::internal::PERCENTILES_LEN;

#[derive(Default)]
pub(crate) struct PercentileResult {
    pub sat_prices: [Cents; PERCENTILES_LEN],
    pub usd_prices: [Cents; PERCENTILES_LEN],
    pub min_price: Cents,
    pub max_price: Cents,
}
