use std::marker::PhantomData;

use bitview_primitives::Count;
use vecdb::BinaryTransform;

pub struct RatioCount<P>(PhantomData<P>);

impl<P: From<f64> + Default> BinaryTransform<Count, Count, P> for RatioCount<P> {
    #[inline(always)]
    fn apply(numerator: Count, denominator: Count) -> P {
        if *denominator > 0 {
            P::from(*numerator as f64 / *denominator as f64)
        } else {
            P::default()
        }
    }
}
