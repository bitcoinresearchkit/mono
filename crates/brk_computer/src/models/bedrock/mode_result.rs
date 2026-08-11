use brk_types::{Cents, StoredF64};

use super::{Percentiles, PriceBands};

pub(super) struct ModeResult {
    pub(super) loss_threshold: Percentiles<StoredF64>,
    pub(super) prices: PriceBands<Cents>,
}
