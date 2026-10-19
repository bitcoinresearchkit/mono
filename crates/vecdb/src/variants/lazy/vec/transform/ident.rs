use super::UnaryTransform;
use crate::{ReadableBoxedVec, ReadableVec, VecIndex, VecValue};

pub struct Ident;

impl<T> UnaryTransform<T> for Ident {
    #[inline(always)]
    fn apply(value: T) -> T {
        value
    }

    fn read_into<I: VecIndex>(
        source: &ReadableBoxedVec<I, T>,
        from: usize,
        to: usize,
        out: &mut Vec<T>,
    ) where
        T: VecValue,
    {
        source.read_into_at(from, to, out);
    }

    fn for_each_chunk<I: VecIndex>(
        source: &ReadableBoxedVec<I, T>,
        from: usize,
        to: usize,
        f: &mut dyn FnMut(usize, &[T]),
    ) where
        T: VecValue,
    {
        source.for_each_chunk_at(from, to, f);
    }
}
