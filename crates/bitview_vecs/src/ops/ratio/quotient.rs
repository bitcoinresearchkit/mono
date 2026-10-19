use std::marker::PhantomData;

use vecdb::BinaryTransform;

/// Numerator over denominator: undefined (NaN, or infinite over zero) when the denominator is zero.
pub struct Quotient<P>(PhantomData<P>);

impl<N, D, P> BinaryTransform<N, D, P> for Quotient<P>
where
    N: Into<f64>,
    D: Into<f64>,
    P: From<f64>,
{
    #[inline(always)]
    fn apply(numerator: N, denominator: D) -> P {
        P::from(numerator.into() / denominator.into())
    }
}
