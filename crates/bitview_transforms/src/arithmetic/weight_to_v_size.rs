use brk_types::{VSize, Weight32};
use vecdb::UnaryTransform;

/// Weight divided by four, rounded up (BIP-141 virtual size).
pub struct WeightToVSize;

impl UnaryTransform<Weight32, VSize> for WeightToVSize {
    #[inline(always)]
    fn apply(weight: Weight32) -> VSize {
        VSize::from(weight)
    }
}
