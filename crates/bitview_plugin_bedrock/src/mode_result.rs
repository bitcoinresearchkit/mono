use bitview_primitives::BoundedRatio;
use brk_types::Cents;

use super::{Percentiles, PriceBands};

pub struct ModeResult {
    pub supply_in_loss_threshold: Percentiles<BoundedRatio>,
    pub prices: PriceBands<Cents>,
}
