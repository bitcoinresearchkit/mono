use std::convert::Infallible;
use std::result::Result;

use super::{super::RawStrategy, ReadOnlyRawVec};
use crate::{
    HEADER_OFFSET, RawMmapSource, ReadWriteRawVec, ReadableVec, VecIndex, VecValue,
    cache::{CachePolicy, Request},
    traits::chunk_folds::for_each_chunk,
};

impl<I, T, S, C: CachePolicy> ReadableVec<I, T> for ReadOnlyRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    #[inline(always)]
    fn collect_one_at(&self, index: usize) -> Option<T> {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.get_source(
                index,
                || (self.base.len(), &[][..]),
                |stored, ranges| {
                    ReadWriteRawVec::<I, T, S, C>::load_cache_ranges(
                        self.base.region(),
                        stored,
                        ranges,
                    )
                },
            );
        }
        let len = self.base.len();
        if index >= len {
            return None;
        }
        Some(self.base.region().with_read_bytes(|bytes| unsafe {
            S::read_from_ptr(bytes.as_ptr().add(HEADER_OFFSET), index * size_of::<T>())
        }))
    }

    fn read_cached_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) -> bool {
        C::cache(&self.cache)
            .is_some_and(|cache| cache.try_read_range(from, to, || self.base.len(), out))
    }

    #[inline(always)]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.read_source(
                Request::Range(from, to),
                buf,
                || (self.base.len(), &[][..]),
                |stored, ranges| {
                    ReadWriteRawVec::<I, T, S, C>::load_cache_ranges(
                        self.base.region(),
                        stored,
                        ranges,
                    )
                },
            );
        }
        ReadWriteRawVec::<I, T, S, C>::read_stored_into(
            self.base.region(),
            self.base.len(),
            from,
            to,
            buf,
        );
    }

    #[inline]
    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.read_source(
                Request::Sorted(indices),
                out,
                || (self.base.len(), &[][..]),
                |stored, ranges| {
                    ReadWriteRawVec::<I, T, S, C>::load_cache_ranges(
                        self.base.region(),
                        stored,
                        ranges,
                    )
                },
            );
        }
        let reader = self.reader();
        out.reserve(indices.len());
        for &index in indices {
            if let Some(value) = reader.try_get_at(index) {
                out.push(value);
            }
        }
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let Some(cache) = C::cache(&self.cache) else {
            if !RawMmapSource::<I, T, S>::try_for_each_chunk(
                self.base.region(),
                self.base.len(),
                from,
                to,
                f,
            ) {
                for_each_chunk(self, from, to, f);
            }
            return;
        };
        cache.for_each_source(
            from,
            to,
            || (self.base.len(), &[][..]),
            |stored, ranges| {
                ReadWriteRawVec::<I, T, S, C>::load_cache_ranges(self.base.region(), stored, ranges)
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
                    ReadWriteRawVec::<I, T, S, C>::load_cache_ranges(
                        self.base.region(),
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
