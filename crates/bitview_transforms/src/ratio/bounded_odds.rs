use bitview_primitives::{BoundedRatio, Ratio64};
use vecdb::{UnaryTransform, unlikely};

/// Odds of a bounded probability; the shared fixed-point scale cancels.
pub struct BoundedOdds;

impl UnaryTransform<BoundedRatio, Ratio64> for BoundedOdds {
    #[inline]
    fn apply(value: BoundedRatio) -> Ratio64 {
        if unlikely(value.is_nan()) {
            Ratio64::new(f64::NAN)
        } else {
            Ratio64::new(value.inner() as f64 / (BoundedRatio::SCALE - value.inner()) as f64)
        }
    }
}
