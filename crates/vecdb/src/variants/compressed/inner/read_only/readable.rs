use crate::{ReadableVec, VecIndex, VecValue};

use super::{
    super::{CompressionStrategy, ReadWriteCompressedVec},
    ReadOnlyCompressedVec,
};

impl<I, T, S> ReadableVec<I, T> for ReadOnlyCompressedVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    #[inline(always)]
    fn cursor_chunk_size(&self) -> usize {
        ReadWriteCompressedVec::<I, T, S>::PER_PAGE
    }

    #[inline(always)]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return;
        }
        buf.reserve(to - from);

        let reader = self.base.region().create_reader();
        let pages = self.pages.read();
        ReadWriteCompressedVec::<I, T, S>::read_stored_pages_into(&reader, &pages, from, to, buf);
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.fold_range_at(from, to, (), |(), v| f(v));
    }

    #[inline]
    fn fold_range_at<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, f: F) -> B
    where
        Self: Sized,
    {
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return init;
        }
        self.fold_source(from, to, len, init, f)
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E>
    where
        Self: Sized,
    {
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return Ok(init);
        }
        self.try_fold_source(from, to, len, init, f)
    }
}
