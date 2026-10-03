use bitview_primitives::{BoundedRatio, StoredF64};
use vecdb::{UnaryTransform, unlikely};

/// Odds of a bounded probability; the shared fixed-point scale cancels.
pub struct BoundedOddsF64;

impl UnaryTransform<BoundedRatio, StoredF64> for BoundedOddsF64 {
    #[inline]
    fn apply(value: BoundedRatio) -> StoredF64 {
        if unlikely(value.is_nan()) {
            StoredF64::NAN
        } else {
            StoredF64::from(value.inner() as f64 / (BoundedRatio::SCALE - value.inner()) as f64)
        }
    }
}
