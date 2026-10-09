use brk_types::{Cents, Height, Timestamp};

use super::PriceRangeMax;

/// Borrowed working data owned by the computation, independent of read-cache retention.
pub struct ComputeContext<'a> {
    pub starting_height: Height,
    pub last_height: Height,
    pub height_to_timestamp: &'a [Timestamp],
    pub height_to_price: &'a [Cents],
    pub price_range_max: &'a PriceRangeMax,
}
