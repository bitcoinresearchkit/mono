use brk_traversable::Traversable;

use super::{SplitCloseByUnit, SplitIndexesByUnit};

#[derive(Clone, Traversable)]
pub struct SplitByUnit {
    pub open: SplitIndexesByUnit,
    pub high: SplitIndexesByUnit,
    pub low: SplitIndexesByUnit,
    pub close: SplitCloseByUnit,
}
