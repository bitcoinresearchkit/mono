use std::marker::PhantomData;

use bitview_primitives::Float32;
use vecdb::BinaryTransform;

pub struct RatioDiffFloat32<P>(PhantomData<P>);

impl<P: From<f64> + Default> BinaryTransform<Float32, Float32, P> for RatioDiffFloat32<P> {
    #[inline(always)]
    fn apply(value: Float32, base: Float32) -> P {
        if base.is_nan() || *base == 0.0 {
            P::default()
        } else {
            P::from((*value / *base - 1.0) as f64)
        }
    }
}
