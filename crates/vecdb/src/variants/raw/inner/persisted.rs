use std::{marker::PhantomData, ops::Range, result::Result};

use rawdb::Region;

use super::RawStrategy;
use crate::{
    HEADER_OFFSET, RawIoSource, RawMmapSource, VecReader, VecValue,
    variants::stored_reads::PersistedReads,
};

pub(crate) struct RawPersisted<'a, T, S> {
    region: &'a Region,
    _marker: PhantomData<(T, S)>,
}

impl<'a, T, S> RawPersisted<'a, T, S> {
    #[inline(always)]
    pub(crate) fn new(region: &'a Region) -> Self {
        Self {
            region,
            _marker: PhantomData,
        }
    }
}

impl<T, S> PersistedReads<T> for RawPersisted<'_, T, S>
where
    T: VecValue,
    S: RawStrategy<T>,
{
    fn load(&self, stored: usize, missing: &[Range<usize>]) -> Vec<(usize, Vec<T>)> {
        missing
            .iter()
            .map(|range| {
                let mut values = Vec::with_capacity(range.len());
                self.read(stored, range.start, range.end, &mut values);
                (range.start, values)
            })
            .collect()
    }

    #[inline]
    fn read(&self, stored: usize, from: usize, to: usize, out: &mut Vec<T>) {
        let to = to.min(stored);
        if from >= to {
            return;
        }
        let offset = HEADER_OFFSET + from * size_of::<T>();
        let bytes = (to - from) * size_of::<T>();
        if self.region.prefers_mmap(offset, bytes) {
            RawMmapSource::<T, S>::new_from_parts(self.region, stored, from, to).read_into(out);
        } else {
            RawIoSource::<T, S>::new_from_parts(self.region, stored, from, to).read_into(out);
        }
    }

    #[inline(always)]
    fn collect_one(&self, _stored: usize, index: usize) -> Option<T> {
        // Borrows only this value: see `ImportOptions::with_unread_updates`. Bounded again under
        // the region lock: a truncation may have won the race since the length read.
        self.region.with_read_range(
            HEADER_OFFSET + index * size_of::<T>(),
            size_of::<T>(),
            |bytes| unsafe { S::read_from_ptr(bytes.as_ptr(), 0) },
        )
    }

    fn read_sorted(&self, stored: usize, indices: &[usize], out: &mut Vec<T>) {
        let reader = VecReader::<usize, T, S>::from_region(self.region, stored);
        for &index in indices {
            if let Some(value) = reader.try_get_at(index) {
                out.push(value);
            }
        }
    }

    fn for_each_chunk(
        &self,
        stored: usize,
        from: usize,
        to: usize,
        f: &mut dyn FnMut(usize, &[T]),
    ) -> bool {
        RawMmapSource::<T, S>::try_for_each_chunk(self.region, stored, from, to, f)
    }

    #[inline(always)]
    fn try_fold<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        stored: usize,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        let offset = HEADER_OFFSET + from * size_of::<T>();
        let bytes = (to - from) * size_of::<T>();
        if self.region.prefers_mmap(offset, bytes) {
            RawMmapSource::<T, S>::new_from_parts(self.region, stored, from, to).try_fold(init, f)
        } else {
            RawIoSource::<T, S>::new_from_parts(self.region, stored, from, to).try_fold(init, f)
        }
    }
}
