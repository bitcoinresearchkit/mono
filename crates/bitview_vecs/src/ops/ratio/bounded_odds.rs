use bitview_primitives::BoundedRatio;
use vecdb::{UnaryTransform, unlikely};

/// Odds of a bounded probability; the shared fixed-point scale cancels. As a ratio at either
/// precision (`Ratio` to publish, `Ratio64` to compute with).
pub struct BoundedOdds;

impl<O: From<f64>> UnaryTransform<BoundedRatio, O> for BoundedOdds {
    #[inline]
    fn apply(value: BoundedRatio) -> O {
        if unlikely(value.is_nan()) {
            O::from(f64::NAN)
        } else {
            O::from(value.inner() as f64 / (BoundedRatio::SCALE - value.inner()) as f64)
        }
    }
}
