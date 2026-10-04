use bitview_primitives::{Count, PerSecond};
use vecdb::UnaryTransform;

/// A count over a window of `SECONDS`, as a rate.
pub struct CountPerSecond<const SECONDS: u32>;

impl<const SECONDS: u32> UnaryTransform<Count, PerSecond> for CountPerSecond<SECONDS> {
    #[inline(always)]
    fn apply(value: Count) -> PerSecond {
        PerSecond::from(u64::from(value) as f64 / SECONDS as f64)
    }
}
