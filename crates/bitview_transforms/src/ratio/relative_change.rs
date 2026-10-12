use std::marker::PhantomData;

use vecdb::BinaryTransform;

/// `value / base - 1`: undefined (NaN, or infinite) when the base is zero.
pub struct RelativeChange<P>(PhantomData<P>);

impl<V, P> BinaryTransform<V, V, P> for RelativeChange<P>
where
    V: Into<f64>,
    P: From<f64>,
{
    #[inline(always)]
    fn apply(value: V, base: V) -> P {
        P::from(value.into() / base.into() - 1.0)
    }
}
