use std::convert::Infallible;
use std::result::Result;

use super::{
    super::{CompressionStrategy, ReadWriteCompressedVec},
    ReadOnlyCompressedVec,
};
use crate::{
    CompressedIoSource, ReadableVec, VecIndex, VecValue,
    cache::{CachePolicy, Request},
    traits::chunk_folds::for_each_chunk,
};

impl<I, T, S, C: CachePolicy> ReadableVec<I, T> for ReadOnlyCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    fn collect_one_at(&self, index: usize) -> Option<T> {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.get_source(
                index,
                || (self.base.len(), &[][..]),
                |stored, ranges| {
                    ReadWriteCompressedVec::<I, T, S, C>::load_cache_ranges(
                        self.base.region(),
                        &self.pages,
                        stored,
                        ranges,
                    )
                },
            );
        }
        let mut values = Vec::with_capacity(1);
        self.read_into_at(index, index.saturating_add(1), &mut values);
        values.pop()
    }

    fn read_cached_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) -> bool {
        C::cache(&self.cache)
            .is_some_and(|cache| cache.try_read_range(from, to, || self.base.len(), out))
    }

    #[inline(always)]
    fn cursor_chunk_size(&self) -> usize {
        ReadWriteCompressedVec::<I, T, S>::PER_PAGE
    }

    #[inline(always)]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.read_source(
                Request::Range(from, to),
                buf,
                || (self.base.len(), &[][..]),
                |stored, ranges| {
                    ReadWriteCompressedVec::<I, T, S, C>::load_cache_ranges(
                        self.base.region(),
                        &self.pages,
                        stored,
                        ranges,
                    )
                },
            );
        }
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return;
        }
        buf.reserve(to - from);

        if ReadWriteCompressedVec::<I, T, S>::prefers_mmap(
            self.base.region(),
            &self.pages,
            from,
            to,
        ) {
            let pages = self.pages.read();
            let reader = self.base.region().create_reader();
            ReadWriteCompressedVec::<I, T, S>::read_stored_pages_into(
                &reader, &pages, from, to, buf,
            );
        } else {
            CompressedIoSource::<I, T, S>::new_from_parts(
                self.base.region(),
                &self.pages,
                len,
                from,
                to,
            )
            .read_into(buf);
        }
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.read_source(
                Request::Sorted(indices),
                out,
                || (self.base.len(), &[][..]),
                |stored, ranges| {
                    ReadWriteCompressedVec::<I, T, S, C>::load_cache_ranges(
                        self.base.region(),
                        &self.pages,
                        stored,
                        ranges,
                    )
                },
            );
        }
        let len = self.base.len();
        let indices = &indices[..indices.partition_point(|&i| i < len)];
        out.reserve(indices.len());
        ReadWriteCompressedVec::<I, T, S>::read_sorted_stored_into(
            self.base.region(),
            &self.pages,
            len,
            indices,
            out,
        );
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let Some(cache) = C::cache(&self.cache) else {
            return for_each_chunk(self, from, to, f);
        };
        cache.for_each_source(
            from,
            to,
            || (self.base.len(), &[][..]),
            |stored, ranges| {
                ReadWriteCompressedVec::<I, T, S, C>::load_cache_ranges(
                    self.base.region(),
                    &self.pages,
                    stored,
                    ranges,
                )
            },
            f,
        );
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
        if C::cache(&self.cache).is_some() {
            let mut f = f;
            return self
                .try_fold_range_at(from, to, init, |acc, value| {
                    Ok::<_, Infallible>(f(acc, value))
                })
                .unwrap();
        }
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return init;
        }
        self.fold_source(from, to, len, init, f)
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E>
    where
        Self: Sized,
    {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.try_fold_source(
                from,
                to,
                init,
                || (self.base.len(), &[][..]),
                |stored, ranges| {
                    ReadWriteCompressedVec::<I, T, S, C>::load_cache_ranges(
                        self.base.region(),
                        &self.pages,
                        stored,
                        ranges,
                    )
                },
                f,
            );
        }
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return Ok(init);
        }
        self.try_fold_source(from, to, len, init, f)
    }
}
