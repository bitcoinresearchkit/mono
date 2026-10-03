use bitview_primitives::{BoundedRatio, StoredF64};
use vecdb::UnaryTransform;

/// Decode a bounded ratio, optionally after its exact encoded complement.
pub struct BoundedToF64<const COMPLEMENT: bool = false>;

impl<const COMPLEMENT: bool> UnaryTransform<BoundedRatio, StoredF64> for BoundedToF64<COMPLEMENT> {
    #[inline]
    fn apply(value: BoundedRatio) -> StoredF64 {
        StoredF64::from(f64::from(if COMPLEMENT {
            value.complement()
        } else {
            value
        }))
    }
}
