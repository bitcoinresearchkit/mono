use std::convert::Infallible;
use std::result::Result;

use super::{super::RawStrategy, ReadWriteRawVec};
use crate::{
    AnyStoredVec, HEADER_OFFSET, READ_CHUNK_SIZE, RawMmapSource, ReadableVec, VecIndex, VecValue,
    cache::CachePolicy, cache::Request, traits::chunk_folds::for_each_chunk,
};

impl<I, T, S, C: CachePolicy> ReadableVec<I, T> for ReadWriteRawVec<I, T, S, C>
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
                || (self.base.stored_len(), self.base.pushed()),
                |stored, ranges| Self::load_cache_ranges(self.base.region(), stored, ranges),
            );
        }
        let len = self.base.len();
        if index >= len {
            return None;
        }

        let stored_len = self.stored_len();
        if index >= stored_len {
            return self.base.pushed().get(index - stored_len).cloned();
        }

        Some(self.base.region().with_read_bytes(|bytes| unsafe {
            S::read_from_ptr(bytes.as_ptr().add(HEADER_OFFSET), index * size_of::<T>())
        }))
    }

    fn read_cached_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) -> bool {
        C::cache(&self.cache)
            .is_some_and(|cache| cache.try_read_range(from, to, || self.base.stored_len(), out))
    }

    #[inline(always)]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.read_source(
                Request::Range(from, to),
                buf,
                || (self.base.stored_len(), self.base.pushed()),
                |stored, ranges| Self::load_cache_ranges(self.base.region(), stored, ranges),
            );
        }
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return;
        }

        buf.reserve(to - from);

        let stored_len = self.stored_len();

        if from < stored_len {
            let stored_to = to.min(stored_len);
            Self::read_stored_into(self.region(), stored_len, from, stored_to, buf);
        }

        if to > stored_len {
            let push_from = from.max(stored_len);
            let pushed = self.base.pushed();
            let start = push_from - stored_len;
            let end = (to - stored_len).min(pushed.len());
            buf.extend_from_slice(&pushed[start..end]);
        }
    }

    #[inline]
    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.read_source(
                Request::Sorted(indices),
                out,
                || (self.base.stored_len(), self.base.pushed()),
                |stored, ranges| Self::load_cache_ranges(self.base.region(), stored, ranges),
            );
        }
        let reader = self.reader();
        let stored_len = reader.stored_len();
        let pushed = self.base.pushed();

        out.reserve(indices.len());
        for &index in indices {
            if index < stored_len {
                out.push(reader.get_at(index));
            } else if let Some(value) = pushed.get(index - stored_len) {
                out.push(value.clone());
            }
        }
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let Some(cache) = C::cache(&self.cache) else {
            let to = to.min(self.base.len());
            if from >= to {
                return;
            }
            let stored = self.stored_len();
            if !RawMmapSource::<I, T, S>::try_for_each_chunk(self.region(), stored, from, to, f) {
                return for_each_chunk(self, from, to, f);
            }
            if to > stored {
                let start = from.max(stored);
                let pushed = &self.base.pushed()[start - stored..to - stored];
                for (index, values) in pushed.chunks(READ_CHUNK_SIZE).enumerate() {
                    f(start + index * READ_CHUNK_SIZE, values);
                }
            }
            return;
        };
        cache.for_each_source(
            from,
            to,
            || (self.base.stored_len(), self.base.pushed()),
            |stored, ranges| Self::load_cache_ranges(self.base.region(), stored, ranges),
            f,
        );
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.fold_range_at(from, to, (), |(), v| f(v));
    }

    #[inline]
    fn fold_range_at<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, mut f: F) -> B
    where
        Self: Sized,
    {
        if C::cache(&self.cache).is_some() {
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

        let stored_len = self.stored_len();

        if to <= stored_len {
            return self.fold_source(from, to, init, f);
        }

        let mut acc = init;
        if from < stored_len {
            acc = self.fold_source(from, stored_len, acc, &mut f);
        }
        self.base.fold_pushed(from, to, acc, f)
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> Result<B, E>
    where
        Self: Sized,
    {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.try_fold_source(
                from,
                to,
                init,
                || (self.base.stored_len(), self.base.pushed()),
                |stored, ranges| Self::load_cache_ranges(self.base.region(), stored, ranges),
                f,
            );
        }
        let len = self.base.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return Ok(init);
        }

        let stored_len = self.stored_len();

        if to <= stored_len {
            return self.try_fold_source(from, to, init, f);
        }

        let mut acc = init;
        if from < stored_len {
            acc = self.try_fold_source(from, stored_len, acc, &mut f)?;
        }
        self.base.try_fold_pushed(from, to, acc, f)
    }
}
