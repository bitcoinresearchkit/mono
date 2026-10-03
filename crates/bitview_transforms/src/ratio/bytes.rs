use std::marker::PhantomData;

use bitview_primitives::Bytes;
use vecdb::BinaryTransform;

pub struct RatioBytes<P>(PhantomData<P>);

impl<P: From<f64> + Default, D: Into<f64>> BinaryTransform<Bytes, D, P> for RatioBytes<P> {
    #[inline(always)]
    fn apply(numerator: Bytes, denominator: D) -> P {
        let denominator: f64 = denominator.into();
        if denominator > 0.0 {
            P::from(f64::from(numerator) / denominator)
        } else {
            P::default()
        }
    }
}
