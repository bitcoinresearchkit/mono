use std::marker::PhantomData;

use vecdb::BinaryTransform;

/// Numerator over denominator, zero when the denominator is zero.
pub struct Quotient<P>(PhantomData<P>);

impl<N, D, P> BinaryTransform<N, D, P> for Quotient<P>
where
    N: Into<f64>,
    D: Into<f64>,
    P: From<f64> + Default,
{
    #[inline(always)]
    fn apply(numerator: N, denominator: D) -> P {
        let denominator = denominator.into();
        if denominator == 0.0 {
            P::default()
        } else {
            P::from(numerator.into() / denominator)
        }
    }
}
