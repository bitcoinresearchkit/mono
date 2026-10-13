use std::ops::Range;
use std::sync::Arc;

use parking_lot::RwLock;
use rawdb::Region;

use crate::{CompressedIoSource, Result, VecIndex, VecValue, cache::CachePolicy};

use super::{CompressionStrategy, Pages, ReadWriteCompressedVec};

impl<I, T, S, C> ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
    C: CachePolicy,
{
    pub(super) fn with_cache_update<R>(
        &mut self,
        from: usize,
        write: impl FnOnce(&mut Self) -> Result<R>,
    ) -> Result<R> {
        C::update(self.cache.clone(), from, || write(self))
    }

    /// Decode each missing physical page once. Adjacent pages become one owned
    /// range; the shared cache decides whether to retain that range or points.
    pub(super) fn load_cache_ranges(
        region: &Region,
        pages: &Arc<RwLock<Pages>>,
        len: usize,
        missing: &[Range<usize>],
    ) -> Vec<(usize, Vec<T>)> {
        let mut ranges: Vec<Range<usize>> = Vec::new();
        for range in missing {
            let from = range.start / Self::PER_PAGE * Self::PER_PAGE;
            let to = range.end.saturating_add(Self::PER_PAGE - 1) / Self::PER_PAGE * Self::PER_PAGE;
            let to = to.min(len);
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
                if Self::prefers_mmap(region, pages, range.start, range.end) {
                    let pages = pages.read_recursive();
                    let reader = region.create_reader();
                    Self::read_stored_pages_into(
                        &reader,
                        &pages,
                        range.start,
                        range.end,
                        &mut values,
                    );
                } else {
                    CompressedIoSource::<I, T, S>::new_from_parts(
                        region,
                        pages,
                        len,
                        range.start,
                        range.end,
                    )
                    .read_into(&mut values);
                }
                (range.start, values)
            })
            .collect()
    }
}
