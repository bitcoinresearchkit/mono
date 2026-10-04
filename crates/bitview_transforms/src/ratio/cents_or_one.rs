use bitview_primitives::Ratio;
use brk_types::Cents;
use vecdb::{BinaryTransform, unlikely};

/// Numerator over denominator; one when the denominator is zero.
pub struct RatioCentsOrOne;

impl BinaryTransform<Cents, Cents, Ratio> for RatioCentsOrOne {
    #[inline(always)]
    fn apply(numerator: Cents, denominator: Cents) -> Ratio {
        let denominator = f64::from(denominator);
        if unlikely(denominator == 0.0) {
            Ratio::new(1.0)
        } else {
            Ratio::from(f64::from(numerator) / denominator)
        }
    }
}
