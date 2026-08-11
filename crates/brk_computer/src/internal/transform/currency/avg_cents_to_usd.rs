use brk_types::{Dollars, StoredF32};
use vecdb::UnaryTransform;

pub struct AvgCentsToUsd;

impl UnaryTransform<StoredF32, Dollars> for AvgCentsToUsd {
    #[inline(always)]
    fn apply(cents: StoredF32) -> Dollars {
        Dollars::from(f64::from(cents) / 100.0)
    }
}
