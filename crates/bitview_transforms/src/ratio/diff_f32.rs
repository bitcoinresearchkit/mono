use std::marker::PhantomData;

use bitview_primitives::StoredF32;
use vecdb::BinaryTransform;

pub struct RatioDiffF32<P>(PhantomData<P>);

impl<P: From<f64> + Default> BinaryTransform<StoredF32, StoredF32, P> for RatioDiffF32<P> {
    #[inline(always)]
    fn apply(value: StoredF32, base: StoredF32) -> P {
        if base.is_nan() || *base == 0.0 {
            P::default()
        } else {
            P::from((*value / *base - 1.0) as f64)
        }
    }
}
