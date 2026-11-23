use bitview_primitives::{BoundedRatio, Percent};
use vecdb::UnaryTransform;

use super::FixedToPercent;

/// Decode a bounded ratio as a percentage, optionally after its exact encoded complement: a share's
/// other side (profit for loss) read from the stored share instead of stored again.
pub struct BoundedToPercent<const COMPLEMENT: bool = false>;

impl<const COMPLEMENT: bool> UnaryTransform<BoundedRatio, Percent>
    for BoundedToPercent<COMPLEMENT>
{
    #[inline(always)]
    fn apply(value: BoundedRatio) -> Percent {
        FixedToPercent::apply(if COMPLEMENT {
            value.complement()
        } else {
            value
        })
    }
}
