use bitview_primitives::{Count, StoredF32};
use vecdb::UnaryTransform;

pub struct PerSecond<const SECONDS: u32>;

impl<const SECONDS: u32> UnaryTransform<Count, StoredF32> for PerSecond<SECONDS> {
    #[inline(always)]
    fn apply(value: Count) -> StoredF32 {
        StoredF32::from(u64::from(value) as f64 / SECONDS as f64)
    }
}
