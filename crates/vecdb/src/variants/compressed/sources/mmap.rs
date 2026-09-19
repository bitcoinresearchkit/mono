use std::{marker::PhantomData, result::Result, sync::Arc};

use parking_lot::{RwLock, RwLockReadGuard};
use rawdb::{Reader, Region};

use super::super::inner::{
    COMPRESSED_PAGE_SIZE, CompressionStrategy, PageDecoder, ReadWriteCompressedVec,
};
use crate::{AnyStoredVec, Pages, VecIndex, VecValue, cache::CachePolicy, unlikely};

/// Read-only mmap-backed source over a compressed vector.
///
/// Only sees **stored** (persisted) values. Pages are decoded lazily —
/// only when fold/for_each reaches them. Consumed by fold/try_fold/for_each.
pub struct CompressedMmapSource<'a, I, T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    reader: Reader,
    pages: RwLockReadGuard<'a, Pages>,
    page_buf: Vec<T>,
    decoder: PageDecoder<T, S>,
    page_buf_idx: usize,
    pos: usize,
    end: usize,
    _marker: PhantomData<(I, T, S)>,
}

impl<'a, I, T, S> CompressedMmapSource<'a, I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    const SIZE_OF_T: usize = size_of::<T>();
    const PER_PAGE: usize = COMPRESSED_PAGE_SIZE / Self::SIZE_OF_T;
    const NO_PAGE: usize = usize::MAX;

    /// Ensures the page at `page_index` is decoded in `page_buf`.
    #[inline(always)]
    fn ensure_page_decoded(&mut self, page_index: usize) -> Option<()> {
        if unlikely(self.page_buf_idx != page_index) {
            self.decode_page_into_buf(page_index)?;
        }
        Some(())
    }

    /// Decode a page into the internal buffer via mmap.
    #[inline(always)]
    fn decode_page_into_buf(&mut self, page_index: usize) -> Option<()> {
        let page = self.pages.get(page_index)?;
        let header = self
            .reader
            .read(page.header_start as usize, page.header_len());
        let body = self.reader.read(page.start as usize, page.bytes as usize);
        let expected_len = page.values_count(Self::PER_PAGE, Self::SIZE_OF_T);
        self.decoder
            .decode_into(page, header, body, expected_len, &mut self.page_buf)
            .ok()?;
        self.page_buf_idx = page_index;
        Some(())
    }

    #[inline(always)]
    pub fn decoded_page(&mut self, page_index: usize) -> Option<&[T]> {
        self.ensure_page_decoded(page_index)?;
        Some(&self.page_buf)
    }

    pub fn new<C: CachePolicy>(
        vec: &'a ReadWriteCompressedVec<I, T, S, C>,
        from: usize,
        to: usize,
    ) -> Self {
        Self::new_from_parts(vec.region(), vec.pages(), vec.stored_len(), from, to)
    }

    pub fn new_from_parts(
        region: &Region,
        pages: &'a Arc<RwLock<Pages>>,
        stored_len: usize,
        from: usize,
        to: usize,
    ) -> Self {
        let from = from.min(stored_len);
        let to = to.min(stored_len);
        Self {
            reader: region.create_reader(),
            pages: pages.read(),
            page_buf: Vec::with_capacity(Self::PER_PAGE),
            decoder: PageDecoder::default(),
            page_buf_idx: Self::NO_PAGE,
            pos: from,
            end: to,
            _marker: PhantomData,
        }
    }

    /// Fold all remaining elements — tight pointer loop per page so LLVM can vectorize.
    #[inline(always)]
    pub fn fold<B, F: FnMut(B, T) -> B>(mut self, init: B, mut f: F) -> B {
        let per_page = Self::PER_PAGE;
        let end = self.end;
        let mut page_index = self.pos / per_page;
        let mut page_start = page_index * per_page;
        let mut in_page_offset = self.pos - page_start;
        let mut accum = init;
        while self.pos < end {
            if self.ensure_page_decoded(page_index).is_none() {
                break;
            }
            let page_end = (end - page_start).min(self.page_buf.len());
            let ptr = self.page_buf.as_ptr();
            let mut index = in_page_offset;
            while index < page_end {
                // Clone rather than move: page_buf still owns and drops every value.
                accum = f(accum, unsafe { (&*ptr.add(index)).clone() });
                index += 1;
            }
            self.pos = page_start + page_end;
            page_index += 1;
            page_start += per_page;
            in_page_offset = 0;
        }
        accum
    }

    /// Fallible fold with early exit on error.
    #[inline(always)]
    pub fn try_fold<B, E, F: FnMut(B, T) -> Result<B, E>>(
        mut self,
        init: B,
        mut f: F,
    ) -> Result<B, E> {
        let per_page = Self::PER_PAGE;
        let end = self.end;
        let mut page_index = self.pos / per_page;
        let mut page_start = page_index * per_page;
        let mut in_page_offset = self.pos - page_start;
        let mut accum = init;
        while self.pos < end {
            if self.ensure_page_decoded(page_index).is_none() {
                break;
            }
            let page_end = (end - page_start).min(self.page_buf.len());
            for value in &self.page_buf[in_page_offset..page_end] {
                accum = f(accum, value.clone())?;
            }
            self.pos = page_start + page_end;
            page_index += 1;
            page_start += per_page;
            in_page_offset = 0;
        }
        Ok(accum)
    }
}
