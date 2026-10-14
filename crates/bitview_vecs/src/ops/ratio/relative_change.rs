use std::marker::PhantomData;

use vecdb::{BinaryTransform, DeltaOp};

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

/// The change from each window's starting value (`LazyDeltaVec`, non-inclusive windows).
impl<V, P> DeltaOp<V, P> for RelativeChange<P>
where
    V: Into<f64>,
    P: From<f64> + Send + Sync + 'static,
{
    #[inline(always)]
    fn combine(current: V, start: V, _count: usize) -> P {
        <Self as BinaryTransform<V, V, P>>::apply(current, start)
    }
}
