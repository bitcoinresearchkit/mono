use std::convert::Infallible;
use std::result::Result;

use super::{super::CompressionStrategy, ReadWriteCompressedVec};
use crate::{
    AnyStoredVec, CompressedIoSource, ReadableVec, VecIndex, VecValue, cache::CachePolicy,
    cache::Request, traits::chunk_folds::for_each_chunk,
};

impl<I, T, S, C: CachePolicy> ReadableVec<I, T> for ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    fn collect_one_at(&self, index: usize) -> Option<T> {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.get_source(
                index,
                || (self.base.stored_len(), self.base.pushed()),
                |stored, ranges| {
                    Self::load_cache_ranges(self.base.region(), &self.pages, stored, ranges)
                },
            );
        }
        let mut values = Vec::with_capacity(1);
        self.read_into_at(index, index.saturating_add(1), &mut values);
        values.pop()
    }

    fn read_cached_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) -> bool {
        C::cache(&self.cache)
            .is_some_and(|cache| cache.try_read_range(from, to, || self.base.stored_len(), out))
    }

    #[inline(always)]
    fn cursor_chunk_size(&self) -> usize {
        Self::PER_PAGE
    }

    #[inline(always)]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.read_source(
                Request::Range(from, to),
                buf,
                || (self.base.stored_len(), self.base.pushed()),
                |stored, ranges| {
                    Self::load_cache_ranges(self.base.region(), &self.pages, stored, ranges)
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
        let stored_len = self.stored_len();

        if from < stored_len {
            let stored_to = to.min(stored_len);
            if Self::prefers_mmap(self.region(), &self.pages, from, stored_to) {
                let pages = self.pages.read();
                let reader = self.create_reader();
                Self::read_stored_pages_into(&reader, &pages, from, stored_to, buf);
            } else {
                CompressedIoSource::new(self, from, stored_to).read_into(buf);
            }
        }

        if to > stored_len {
            let push_from = from.max(stored_len);
            let pushed = self.base.pushed();
            let start = push_from - stored_len;
            let end = (to - stored_len).min(pushed.len());
            buf.extend_from_slice(&pushed[start..end]);
        }
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        if let Some(cache) = C::cache(&self.cache) {
            return cache.read_source(
                Request::Sorted(indices),
                out,
                || (self.base.stored_len(), self.base.pushed()),
                |stored, ranges| {
                    Self::load_cache_ranges(self.base.region(), &self.pages, stored, ranges)
                },
            );
        }
        let len = self.base.len();
        let indices = &indices[..indices.partition_point(|&i| i < len)];
        let stored_len = self.stored_len();
        let split = indices.partition_point(|&i| i < stored_len);
        out.reserve(indices.len());
        Self::read_sorted_stored_into(
            self.region(),
            &self.pages,
            stored_len,
            &indices[..split],
            out,
        );
        if split < indices.len() {
            let pushed = self.base.pushed();
            out.extend(
                indices[split..]
                    .iter()
                    .map(|&i| pushed[i - stored_len].clone()),
            );
        }
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let Some(cache) = C::cache(&self.cache) else {
            return for_each_chunk(self, from, to, f);
        };
        cache.for_each_source(
            from,
            to,
            || (self.base.stored_len(), self.base.pushed()),
            |stored, ranges| {
                Self::load_cache_ranges(self.base.region(), &self.pages, stored, ranges)
            },
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
                |stored, ranges| {
                    Self::load_cache_ranges(self.base.region(), &self.pages, stored, ranges)
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
