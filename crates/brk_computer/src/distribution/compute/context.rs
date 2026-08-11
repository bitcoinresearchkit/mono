use brk_types::{Cents, Height, Timestamp};
use vecdb::VecIndex;

use super::PriceRangeMax;

pub struct ComputeContext<'a> {
    pub starting_height: Height,
    pub last_height: Height,
    pub height_to_timestamp: &'a [Timestamp],
    pub height_to_price: &'a [Cents],
    pub price_range_max: &'a PriceRangeMax,
}

impl<'a> ComputeContext<'a> {
    pub(crate) fn price_at(&self, height: Height) -> Cents {
        self.height_to_price[height.to_usize()]
    }

    pub(crate) fn timestamp_at(&self, height: Height) -> Timestamp {
        self.height_to_timestamp[height.to_usize()]
    }
}
