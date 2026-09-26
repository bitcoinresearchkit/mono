use std::{ptr::NonNull, sync::Arc};

use parking_lot::RwLock;
use rawdb::Region;

use crate::{Pages, VecIndex, VecValue, unlikely};

use super::super::inner::{COMPRESSED_PAGE_SIZE, CompressionStrategy, ReadWriteCompressedVec};
use super::{CompressedIoSource, CompressedMmapSource};

enum Owner<'a, I, T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    Mmap(CompressedMmapSource<'a, I, T, S>),
    Io(CompressedIoSource<'a, I, T, S>),
}

#[cfg(all(test, feature = "pco"))]
mod tests {
    use super::{
        CompressedIoSource, CompressedMmapSource, CompressedRangeCursor, CursorState, Owner,
    };
    use crate::{AnyStoredVec, Database, ImportableVec, PcoVec, Version, WritableVec};
    use tempfile::tempdir;

    #[test]
    fn sorted_gather_matches_forced_io_and_mmap_decoders() {
        let directory = tempdir().unwrap();
        let db = Database::open(directory.path()).unwrap();
        let mut source = PcoVec::<usize, u64>::import(&db, "gather", Version::ONE).unwrap();
        let expected: Vec<_> = (0..20_137u64).map(|i| i * 7).collect();
        for &value in &expected {
            source.push(value);
        }
        source.write().unwrap();
        let indices = [0, 0, 7, 1023, 1024, 4096, 19_999, 20_136];
        for owner in [
            Owner::Io(CompressedIoSource::new(&source, 0, expected.len())),
            Owner::Mmap(CompressedMmapSource::new(&source, 0, expected.len())),
        ] {
            let cursor = CompressedRangeCursor {
                owner,
                state: CursorState::new(0, expected.len()),
            };
            let mut actual = vec![99];
            cursor.read_sorted_into(&indices, &mut actual);
            assert_eq!(&actual[1..], indices.map(|i| expected[i]));
        }
    }

    #[test]
    fn bulk_collection_keeps_io_and_mmap_cursors_ready_for_scalar_reads() {
        let directory = tempdir().unwrap();
        let db = Database::open(directory.path()).unwrap();
        let mut source = PcoVec::<usize, u64>::import(&db, "bulk", Version::ONE).unwrap();
        let expected: Vec<_> = (0..20_137u64).map(|i| i * 7).collect();
        for &value in &expected {
            source.push(value);
        }
        source.write().unwrap();
        for owner in [
            Owner::Io(CompressedIoSource::new(&source, 997, expected.len())),
            Owner::Mmap(CompressedMmapSource::new(&source, 997, expected.len())),
        ] {
            let mut cursor = CompressedRangeCursor {
                owner,
                state: CursorState::new(997, expected.len()),
            };
            let mut actual = vec![99];
            cursor.collect_into(4_003, &mut actual);
            assert_eq!(actual, expected[997..5_000]);
            assert_eq!(cursor.next(), Some(expected[5_000]));
            assert_eq!(
                cursor.fold(7, 0, |sum, value| sum + value),
                expected[5_001..5_008].iter().sum::<u64>()
            );
            cursor.collect_into(usize::MAX, &mut actual);
            assert_eq!(actual, expected[5_008..]);
            assert_eq!(cursor.next(), None);
        }
    }
}

struct CursorState<T> {
    position: usize,
    end: usize,
    page_index: usize,
    page: *const T,
    page_len: usize,
}

impl<T: VecValue> CursorState<T> {
    const PER_PAGE: usize = COMPRESSED_PAGE_SIZE / size_of::<T>();
    const NO_PAGE: usize = usize::MAX;

    fn new(position: usize, end: usize) -> Self {
        Self {
            position,
            end,
            page_index: Self::NO_PAGE,
            page: NonNull::dangling().as_ptr(),
            page_len: 0,
        }
    }

    #[inline(always)]
    fn remaining(&self) -> usize {
        self.end - self.position
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.position = self.position.saturating_add(n).min(self.end);
    }

    #[inline(always)]
    fn page_index(&self) -> usize {
        self.position / Self::PER_PAGE
    }

    #[inline(always)]
    fn has_page(&self, page_index: usize) -> bool {
        self.page_index == page_index
    }

    #[inline(always)]
    fn set_page(&mut self, page_index: usize, page: *const T, page_len: usize) {
        self.page_index = page_index;
        self.page = page;
        self.page_len = page_len;
    }

    #[inline(always)]
    fn next(&mut self) -> T {
        let local = self.position - self.page_index * Self::PER_PAGE;
        debug_assert!(local < self.page_len);
        // SAFETY: the cursor refreshes this pointer before entering a new page.
        let value = unsafe { (&*self.page.add(local)).clone() };
        self.position += 1;
        value
    }

    #[inline(always)]
    fn contains_until(&self, end: usize) -> bool {
        if self.page_index == Self::NO_PAGE {
            return false;
        }
        let page_start = self.page_index * Self::PER_PAGE;
        self.position >= page_start && end <= page_start + self.page_len
    }

    #[inline(always)]
    fn fold_buffered_until<B>(
        &mut self,
        end: usize,
        mut value: B,
        fold: &mut impl FnMut(B, T) -> B,
    ) -> B {
        let page_start = self.page_index * Self::PER_PAGE;
        debug_assert!(self.position >= page_start);
        debug_assert!(end >= self.position);
        debug_assert!(end - page_start <= self.page_len);
        let mut local = self.position - page_start;
        let local_end = end - page_start;
        while local < local_end {
            // SAFETY: local is bounded by the current decoded page length.
            value = fold(value, unsafe { (&*self.page.add(local)).clone() });
            local += 1;
        }
        self.position = end;
        value
    }

    #[inline(always)]
    fn fold_with<B>(
        &mut self,
        end: usize,
        mut value: B,
        mut fold: impl FnMut(B, T) -> B,
        mut decode_page: impl FnMut(usize) -> Option<(*const T, usize)>,
    ) -> B {
        while self.position < end {
            let page_index = self.page_index();
            if unlikely(!self.has_page(page_index)) {
                let (page, page_len) = match decode_page(page_index) {
                    Some(page) => page,
                    None => break,
                };
                self.set_page(page_index, page, page_len);
            }
            let page_start = page_index * Self::PER_PAGE;
            let page_end = (end - page_start).min(self.page_len);
            let mut local = self.position - page_start;
            while local < page_end {
                // SAFETY: local is bounded by the current decoded page length.
                value = fold(value, unsafe { (&*self.page.add(local)).clone() });
                local += 1;
            }
            self.position = page_start + page_end;
        }
        value
    }
}

/// Forward cursor over a bounded persisted range of a compressed vector.
///
/// The declared range lets the cursor choose once between mmap for resident
/// pages and buffered file I/O for a cold sequential scan.
pub struct CompressedRangeCursor<'a, I, T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    owner: Owner<'a, I, T, S>,
    state: CursorState<T>,
}

impl<'a, I, T, S> CompressedRangeCursor<'a, I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    /// Returns the current absolute vector position.
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.state.position
    }

    /// Returns the number of values remaining in the declared range.
    #[inline(always)]
    pub fn remaining(&self) -> usize {
        self.state.remaining()
    }

    /// Advances within the declared range without decoding values.
    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.state.advance(n);
    }

    #[inline(always)]
    fn ensure_page(&mut self) -> Option<()> {
        let page_index = self.state.page_index();
        if unlikely(!self.state.has_page(page_index)) {
            let page = match &mut self.owner {
                Owner::Mmap(source) => source.decoded_page(page_index)?,
                Owner::Io(source) => source.decoded_page(page_index)?,
            };
            self.state.set_page(page_index, page.as_ptr(), page.len());
        }
        Some(())
    }

    /// Returns the next value and advances the position.
    #[inline(always)]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<T> {
        if self.state.position >= self.state.end {
            return None;
        }
        self.ensure_page()?;
        Some(self.state.next())
    }

    /// Folds over up to the next `n` values and advances the position.
    #[inline]
    pub fn fold<B>(&mut self, n: usize, value: B, mut fold: impl FnMut(B, T) -> B) -> B {
        let end = self.state.position.saturating_add(n).min(self.state.end);
        if self.state.contains_until(end) {
            return self.state.fold_buffered_until(end, value, &mut fold);
        }

        match &mut self.owner {
            Owner::Mmap(source) => self.state.fold_with(end, value, fold, |page_index| {
                let page = source.decoded_page(page_index)?;
                Some((page.as_ptr(), page.len()))
            }),
            Owner::Io(source) => self.state.fold_with(end, value, fold, |page_index| {
                let page = source.decoded_page(page_index)?;
                Some((page.as_ptr(), page.len()))
            }),
        }
    }

    /// Calls `f` for up to the next `n` values and advances the position.
    #[inline]
    pub fn for_each(&mut self, n: usize, mut f: impl FnMut(T)) {
        self.fold(n, (), |(), value| f(value));
    }

    /// Collects up to the next `n` values into reusable scratch, clearing it first.
    pub fn collect_into(&mut self, n: usize, out: &mut Vec<T>) {
        out.clear();
        let end = self.state.position.saturating_add(n).min(self.state.end);
        out.reserve(end - self.state.position);
        while self.state.position < end {
            let page_index = self.state.page_index();
            let page = match &mut self.owner {
                Owner::Mmap(source) => source.decoded_page(page_index),
                Owner::Io(source) => source.decoded_page(page_index),
            };
            let Some(page) = page else { break };
            let page_start = page_index * CursorState::<T>::PER_PAGE;
            let from = self.state.position - page_start;
            let to = (end - page_start).min(page.len());
            self.state.set_page(page_index, page.as_ptr(), page.len());
            out.extend_from_slice(&page[from..to]);
            self.state.position = page_start + to;
        }
    }

    /// Consume a sorted request directly from decoded pages, without copying
    /// each page through the generic Cursor buffer. Consumes this cursor so no
    /// buffered pointer survives a decoder refill performed by this method.
    pub(crate) fn read_sorted_into(mut self, mut indices: &[usize], out: &mut Vec<T>) {
        while let Some(&first) = indices.first() {
            let page_index = first / CursorState::<T>::PER_PAGE;
            let count = indices.partition_point(|&i| i / CursorState::<T>::PER_PAGE == page_index);
            let page = match &mut self.owner {
                Owner::Mmap(source) => source.decoded_page(page_index),
                Owner::Io(source) => source.decoded_page(page_index),
            }
            .expect("requested compressed page must exist");
            let base = page_index * CursorState::<T>::PER_PAGE;
            out.extend(indices[..count].iter().map(|&i| page[i - base].clone()));
            indices = &indices[count..];
        }
    }

    pub fn new(
        region: &'a Region,
        pages: &'a Arc<RwLock<Pages>>,
        stored_len: usize,
        from: usize,
        to: usize,
    ) -> Self {
        let from = from.min(stored_len);
        let to = to.min(stored_len).max(from);
        let owner = if ReadWriteCompressedVec::<I, T, S>::prefers_mmap(region, pages, from, to) {
            Owner::Mmap(CompressedMmapSource::new_from_parts(
                region, pages, stored_len, from, to,
            ))
        } else {
            Owner::Io(CompressedIoSource::new_from_parts(
                region, pages, stored_len, from, to,
            ))
        };
        Self {
            owner,
            state: CursorState::new(from, to),
        }
    }
}
