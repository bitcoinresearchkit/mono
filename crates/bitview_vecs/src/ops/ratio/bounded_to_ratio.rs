use bitview_primitives::{BoundedRatio, Ratio64};
use vecdb::UnaryTransform;

/// Decode a bounded ratio, optionally after its exact encoded complement.
pub struct BoundedToRatio<const COMPLEMENT: bool = false>;

impl<const COMPLEMENT: bool> UnaryTransform<BoundedRatio, Ratio64> for BoundedToRatio<COMPLEMENT> {
    #[inline]
    fn apply(value: BoundedRatio) -> Ratio64 {
        Ratio64::new(f64::from(if COMPLEMENT {
            value.complement()
        } else {
            value
        }))
    }
}
