use std::marker::PhantomData;

use bitview_primitives::StoredU64;
use vecdb::BinaryTransform;

pub struct RatioU64<P>(PhantomData<P>);

impl<P: From<f64> + Default> BinaryTransform<StoredU64, StoredU64, P> for RatioU64<P> {
    #[inline(always)]
    fn apply(numerator: StoredU64, denominator: StoredU64) -> P {
        if *denominator > 0 {
            P::from(*numerator as f64 / *denominator as f64)
        } else {
            P::default()
        }
    }
}
