use std::{marker::PhantomData, ops::Range, result::Result, sync::Arc};

use parking_lot::RwLock;
use rawdb::Region;

use super::{CompressionStrategy, Pages, ReadWriteCompressedVec};
use crate::{
    CompressedIoSource, CompressedMmapSource, CompressedRangeCursor, VecValue,
    variants::stored_reads::PersistedReads,
};

/// A compressed vector's persisted values: its region and page table. Each read takes the
/// pages before the region (the writer holds the pages while it rewrites the region), mapped
/// when resident, else through buffered I/O.
pub(crate) struct CompressedPersisted<'a, T, S> {
    region: &'a Region,
    pages: &'a Arc<RwLock<Pages>>,
    _marker: PhantomData<(T, S)>,
}

impl<'a, T, S> CompressedPersisted<'a, T, S> {
    #[inline(always)]
    pub(crate) fn new(region: &'a Region, pages: &'a Arc<RwLock<Pages>>) -> Self {
        Self {
            region,
            pages,
            _marker: PhantomData,
        }
    }
}

impl<T, S> CompressedPersisted<'_, T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    const PER_PAGE: usize = ReadWriteCompressedVec::<usize, T, S>::PER_PAGE;

    #[inline(always)]
    fn prefers_mmap(&self, from: usize, to: usize) -> bool {
        ReadWriteCompressedVec::<usize, T, S>::prefers_mmap(self.region, self.pages, from, to)
    }
}

impl<T, S> PersistedReads<T> for CompressedPersisted<'_, T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    /// Decodes each missing physical page once. Adjacent pages become one owned range; the
    /// shared cache decides whether to retain that range or points.
    fn load(&self, stored: usize, missing: &[Range<usize>]) -> Vec<(usize, Vec<T>)> {
        let mut ranges: Vec<Range<usize>> = Vec::new();
        for range in missing {
            let from = range.start / Self::PER_PAGE * Self::PER_PAGE;
            let to = range.end.saturating_add(Self::PER_PAGE - 1) / Self::PER_PAGE * Self::PER_PAGE;
            let to = to.min(stored);
            if let Some(last) = ranges.last_mut()
                && from <= last.end
            {
                last.end = last.end.max(to);
            } else {
                ranges.push(from..to)
            }
        }
        ranges
            .into_iter()
            .map(|range| {
                let mut values = Vec::with_capacity(range.len());
                self.read(stored, range.start, range.end, &mut values);
                (range.start, values)
            })
            .collect()
    }

    fn read(&self, stored: usize, from: usize, to: usize, out: &mut Vec<T>) {
        let to = to.min(stored);
        if from >= to {
            return;
        }
        if self.prefers_mmap(from, to) {
            CompressedMmapSource::<T, S>::new_from_parts(self.region, self.pages, stored, from, to)
                .read_into(out)
        } else {
            CompressedIoSource::<T, S>::new_from_parts(self.region, self.pages, stored, from, to)
                .read_into(out)
        }
        .expect("compressed page decode failed");
    }

    fn collect_one(&self, stored: usize, index: usize) -> Option<T> {
        let mut values = Vec::with_capacity(1);
        self.read(stored, index, index + 1, &mut values);
        values.pop()
    }

    /// Batches adjacent requested pages. Disjoint runs are separated before choosing mmap
    /// versus buffered I/O, so sparse reads do not prefetch unused gaps.
    fn read_sorted(&self, stored: usize, mut indices: &[usize], out: &mut Vec<T>) {
        while let Some(&first) = indices.first() {
            let mut count = 1;
            let mut last_page = first / Self::PER_PAGE;
            while count < indices.len() {
                let page = indices[count] / Self::PER_PAGE;
                if page > last_page + 1 {
                    break;
                }
                last_page = page;
                count += 1;
            }
            let (run, rest) = indices.split_at(count);
            CompressedRangeCursor::<T, S>::new(
                self.region,
                self.pages,
                stored,
                first,
                run[count - 1] + 1,
            )
            .read_sorted_into(run, out);
            indices = rest;
        }
    }

    /// One decoded page at a time, through one decoder.
    fn for_each_chunk(
        &self,
        stored: usize,
        from: usize,
        to: usize,
        f: &mut dyn FnMut(usize, &[T]),
    ) -> bool {
        let to = to.min(stored);
        if from >= to {
            return true;
        }
        if self.prefers_mmap(from, to) {
            CompressedMmapSource::<T, S>::new_from_parts(self.region, self.pages, stored, from, to)
                .for_each_chunk(f);
        } else {
            CompressedIoSource::<T, S>::new_from_parts(self.region, self.pages, stored, from, to)
                .for_each_chunk(f);
        }
        true
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
        if self.prefers_mmap(from, to) {
            CompressedMmapSource::<T, S>::new_from_parts(self.region, self.pages, stored, from, to)
                .try_fold(init, f)
        } else {
            CompressedIoSource::<T, S>::new_from_parts(self.region, self.pages, stored, from, to)
                .try_fold(init, f)
        }
    }
}
