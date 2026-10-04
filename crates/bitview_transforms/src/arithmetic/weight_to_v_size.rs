use bitview_primitives::Weight64;
use brk_types::{VSize, Weight};
use vecdb::UnaryTransform;

/// Weight divided by four, rounded up (BIP-141 virtual size).
pub struct WeightToVSize;

impl UnaryTransform<Weight, VSize> for WeightToVSize {
    #[inline(always)]
    fn apply(weight: Weight) -> VSize {
        VSize::from(weight)
    }
}

impl UnaryTransform<Weight64, VSize> for WeightToVSize {
    #[inline(always)]
    fn apply(weight: Weight64) -> VSize {
        VSize::new(u64::from(weight).div_ceil(4))
    }
}
