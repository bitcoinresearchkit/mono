use std::marker::PhantomData;

use crate::UnaryTransform;

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

#[cfg(test)]
mod tests {
    use super::*;
    struct AddOne;

    impl UnaryTransform<usize> for AddOne {
        fn apply(value: usize) -> usize {
            value + 1
        }
    }

    #[test]
    fn maps_present_values_and_preserves_absence() {
        assert_eq!(MapOption::<AddOne>::apply(Some(5usize)), Some(6));
        assert_eq!(MapOption::<AddOne>::apply(None::<usize>), None);
    }
}
