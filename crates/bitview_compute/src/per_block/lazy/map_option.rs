use std::marker::PhantomData;

use vecdb::UnaryTransform;

pub struct MapOption<F>(PhantomData<F>);

impl<F, S, T> UnaryTransform<Option<S>, Option<T>> for MapOption<F>
where
    F: UnaryTransform<S, T>,
{
    #[inline(always)]
    fn apply(value: Option<S>) -> Option<T> {
        value.map(F::apply)
    }
}
