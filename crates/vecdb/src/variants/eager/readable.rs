use std::result::Result;

use super::EagerVec;
use crate::{ReadableVec, StoredVec};

impl<V> ReadableVec<V::I, V::T> for EagerVec<V>
where
    V: StoredVec,
{
    fn read_cached_into_at(&self, from: usize, to: usize, out: &mut Vec<V::T>) -> bool {
        self.0.read_cached_into_at(from, to, out)
    }

    #[inline(always)]
    fn cursor_chunk_size(&self) -> usize {
        self.0.cursor_chunk_size()
    }

    #[inline(always)]
    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[V::T])) {
        self.0.for_each_chunk_at(from, to, f);
    }

    #[inline(always)]
    fn collect_one_at(&self, index: usize) -> Option<V::T> {
        self.0.collect_one_at(index)
    }

    #[inline(always)]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<V::T>) {
        self.0.read_into_at(from, to, buf)
    }

    #[inline(always)]
    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<V::T>) {
        self.0.read_sorted_into_at(indices, out)
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(V::T)) {
        self.0.for_each_range_dyn_at(from, to, f)
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, V::T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E>
    where
        Self: Sized,
    {
        self.0.try_fold_range_at(from, to, init, f)
    }
}
