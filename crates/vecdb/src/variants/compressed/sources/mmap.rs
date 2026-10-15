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
/// only when fold/for_each reaches them. Consumed by try_fold/for_each.
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
        // Our own data: a decode failure is corruption, never the end of the range.
        self.decoder
            .decode_into(page, header, body, expected_len, &mut self.page_buf)
            .expect("compressed page decode failed");
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
        // Pages before the region: the writer holds the pages while it rewrites the region.
        let pages = pages.read_recursive();
        let reader = region.create_reader();
        let stored_len = stored_len.min(pages.stored_len(Self::PER_PAGE, Self::SIZE_OF_T));
        let from = from.min(stored_len);
        let to = to.min(stored_len);
        Self {
            reader,
            pages,
            page_buf: Vec::with_capacity(Self::PER_PAGE),
            decoder: PageDecoder::default(),
            page_buf_idx: Self::NO_PAGE,
            pos: from,
            end: to,
            _marker: PhantomData,
        }
    }

    /// Lends the rest of the next page within the range with its first index, advancing past it.
    #[inline(always)]
    fn next_chunk(&mut self) -> Option<(usize, &[T])> {
        if self.pos >= self.end {
            return None;
        }
        let page_index = self.pos / Self::PER_PAGE;
        let page_start = page_index * Self::PER_PAGE;
        self.ensure_page_decoded(page_index)?;
        let from = self.pos - page_start;
        let to = (self.end - page_start).min(self.page_buf.len());
        if from >= to {
            return None;
        }
        let at = self.pos;
        self.pos = page_start + to;
        Some((at, &self.page_buf[from..to]))
    }

    /// Visits the range one decoded page at a time.
    pub fn for_each_chunk(mut self, f: &mut dyn FnMut(usize, &[T])) {
        while let Some((at, values)) = self.next_chunk() {
            f(at, values);
        }
    }

    /// Fallible fold with early exit on error.
    #[inline(always)]
    pub fn try_fold<B, E, F: FnMut(B, T) -> Result<B, E>>(
        mut self,
        init: B,
        mut f: F,
    ) -> Result<B, E> {
        let mut accum = init;
        while let Some((_, values)) = self.next_chunk() {
            accum = values.iter().cloned().try_fold(accum, &mut f)?;
        }
        Ok(accum)
    }
}
