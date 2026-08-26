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
    pub(crate) fn new(
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
}
