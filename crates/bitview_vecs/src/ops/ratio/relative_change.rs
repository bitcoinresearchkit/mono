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

/// The change from the value before each window (`LazyDeltaVec`): over a trailing window, the represented
/// value against the last one older than the window, like the window deltas. Undefined before the first value.
impl<V, P> DeltaOp<V, P> for RelativeChange<P>
where
    V: Into<f64> + Default,
    P: From<f64> + Send + Sync + 'static,
{
    #[inline(always)]
    fn ago_index(start: usize) -> Option<usize> {
        start.checked_sub(1)
    }

    #[inline(always)]
    fn ago_default() -> V {
        V::default()
    }

    #[inline(always)]
    fn combine(current: V, before: V, _count: usize) -> P {
        <Self as BinaryTransform<V, V, P>>::apply(current, before)
    }
}
