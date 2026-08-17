use brk_traversable::Traversable;

use super::{SplitCloseByUnit, SplitIndexesByUnit};

#[derive(Clone, Traversable)]
pub struct SplitByUnit {
    /// Opening Bitcoin price for each supported time period, including daily
    /// periods. A populated period uses its first block-level price; an empty
    /// period carries forward the previous close.
    pub open: SplitIndexesByUnit,
    /// Highest Bitcoin price for each supported time period, including daily
    /// periods. A populated period uses its maximum block-level price; an empty
    /// period carries forward the previous close.
    pub high: SplitIndexesByUnit,
    /// Lowest Bitcoin price for each supported time period, including daily
    /// periods. A populated period uses its minimum block-level price; an empty
    /// period carries forward the previous close.
    pub low: SplitIndexesByUnit,
    /// Closing Bitcoin price for each supported time period, including daily
    /// periods. A populated period uses its final block-level price; an empty
    /// period is null.
    pub close: SplitCloseByUnit,
}
