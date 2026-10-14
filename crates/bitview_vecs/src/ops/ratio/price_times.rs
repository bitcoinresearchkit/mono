use std::marker::PhantomData;

use brk_types::Cents;
use vecdb::BinaryTransform;

pub struct PriceTimesRatio<R>(PhantomData<R>);

impl<R: Into<f64>> BinaryTransform<Cents, R, Cents> for PriceTimesRatio<R> {
    #[inline(always)]
    fn apply(price: Cents, ratio: R) -> Cents {
        Cents::from(f64::from(price) * ratio.into())
    }
}
