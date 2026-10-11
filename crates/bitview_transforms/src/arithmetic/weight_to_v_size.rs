use brk_types::Weight;
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

impl UnaryTransform<Weight, VSize> for WeightToVSize {
    #[inline(always)]
    fn apply(weight: Weight) -> VSize {
        VSize::new(u64::from(weight).div_ceil(4))
    }
}
