use std::marker::PhantomData;

use vecdb::BinaryTransform;

/// `value / base - 1`, zero when the base is zero.
pub struct RelativeChange<P>(PhantomData<P>);

impl<V, P> BinaryTransform<V, V, P> for RelativeChange<P>
where
    V: Into<f64>,
    P: From<f64> + Default,
{
    #[inline(always)]
    fn apply(value: V, base: V) -> P {
        let base = base.into();
        if base == 0.0 {
            P::default()
        } else {
            P::from(value.into() / base - 1.0)
        }
    }
}
