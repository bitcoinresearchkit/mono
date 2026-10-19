use brk_types::Cents;
use vecdb::BinaryTransform;

pub struct PriceTimesRatio;

impl<R: Into<f64>> BinaryTransform<Cents, R, Cents> for PriceTimesRatio {
    #[inline(always)]
    fn apply(price: Cents, ratio: R) -> Cents {
        Cents::from(f64::from(price) * ratio.into())
    }
}
