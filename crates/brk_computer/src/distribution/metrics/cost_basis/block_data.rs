use brk_types::{Cents, PartsPerMillion32};

use crate::internal::PERCENTILES_LEN;

#[derive(Clone)]
pub(crate) struct CostBasisBlockData {
    pub min: Cents,
    pub max: Cents,
    pub per_coin: [Cents; PERCENTILES_LEN],
    pub per_dollar: [Cents; PERCENTILES_LEN],
    pub supply_density: PartsPerMillion32,
}
