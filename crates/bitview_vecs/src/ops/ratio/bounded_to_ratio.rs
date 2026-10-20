use bitview_primitives::BoundedRatio;
use vecdb::UnaryTransform;

/// Decode a bounded ratio, optionally after its exact encoded complement, as a ratio at either
/// precision (`Ratio` to publish, `Ratio64` to compute with).
pub struct BoundedToRatio<const COMPLEMENT: bool = false>;

impl<const COMPLEMENT: bool, O: From<f64>> UnaryTransform<BoundedRatio, O>
    for BoundedToRatio<COMPLEMENT>
{
    #[inline]
    fn apply(value: BoundedRatio) -> O {
        O::from(f64::from(if COMPLEMENT {
            value.complement()
        } else {
            value
        }))
    }
}
