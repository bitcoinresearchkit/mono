use std::marker::PhantomData;

use crate::BinaryTransform;

/// Apply a binary transform with the input order reversed.
pub struct ReverseOperands<F>(PhantomData<F>);

impl<S, D, T, F> BinaryTransform<S, D, T> for ReverseOperands<F>
where
    F: BinaryTransform<D, S, T>,
{
    #[inline]
    fn apply(source: S, operand: D) -> T {
        F::apply(operand, source)
    }
}
