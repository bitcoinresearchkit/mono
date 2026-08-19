use brk_types::{Cents, StoredF64};

use super::{Percentiles, PriceBands};

pub struct ModeResult {
    pub loss_threshold: Percentiles<StoredF64>,
    pub prices: PriceBands<Cents>,
}
