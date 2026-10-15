use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    marker::PhantomData,
    mem,
    result::Result,
    sync::Arc,
};

use parking_lot::{RwLock, RwLockReadGuard};
use rawdb::Region;

use super::super::inner::{
    COMPRESSED_PAGE_SIZE, CompressionStrategy, Page, PageDecoder, ReadWriteCompressedVec,
};
use crate::{AnyStoredVec, BUFFER_SIZE, Pages, VecIndex, VecValue, cache::CachePolicy, unlikely};

/// Buffered file I/O source for reading stored compressed data.
///
/// Uses dedicated file handle for sequential reads (better OS readahead than mmap).
/// Only sees stored (persisted) values. Consumed by try_fold/for_each.
pub struct CompressedIoSource<'a, I, T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    file: File,
    file_position: u64,
    region_start: u64,
    buffer: Vec<u8>,
    buffer_len: usize,
    buffer_start_offset: u64,
    decoded_values: Vec<T>,
    decoder: PageDecoder<T, S>,
    decoded_page_index: usize,
    pages: RwLockReadGuard<'a, Pages>,
    index: usize,
    end_index: usize,
    _region_lock: RwLockReadGuard<'a, ()>,
    _marker: PhantomData<(I, T, S)>,
}

impl<'a, I, T, S> CompressedIoSource<'a, I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    const SIZE_OF_T: usize = size_of::<T>();
    const PER_PAGE: usize = COMPRESSED_PAGE_SIZE / Self::SIZE_OF_T;
    const NO_PAGE: usize = usize::MAX;

    #[inline(always)]
    fn ensure_page_decoded(&mut self, page_index: usize) -> Option<()> {
        if unlikely(self.decoded_page_index != page_index) {
            self.decode_page(page_index)?;
        }
        Some(())
    }

    fn refill_buffer(&mut self, starting_page_index: usize) -> Option<()> {
        let start_page = self.pages.get(starting_page_index)?;
        let start_offset = start_page.header_start;

        let last_needed_page = if self.end_index == 0 {
            0
        } else {
            (self.end_index - 1) / Self::PER_PAGE
        };
        let max_page = last_needed_page.min(self.pages.len().saturating_sub(1));

        let mut end_offset = start_offset;
        for i in starting_page_index..=max_page {
            let page = self.pages.get(i)?;
            let candidate_end = page.end();
            if candidate_end.checked_sub(start_offset)? as usize > BUFFER_SIZE {
                break;
            }
            end_offset = candidate_end;
        }
        let total_bytes = end_offset.checked_sub(start_offset)? as usize;

        if total_bytes == 0 {
            return None;
        }

        let absolute_offset = self.region_start + start_offset;
        if self.file_position != absolute_offset {
            self.file_position = absolute_offset;
            self.file.seek(SeekFrom::Start(absolute_offset)).unwrap();
        }

        if self.buffer.capacity() == 0 {
            // Once, for the whole requested span: later refills start further on and never
            // need more, so the buffer neither reallocates nor zeroes bytes it never reads.
            let span = self.pages.get(max_page)?.end().checked_sub(start_offset)? as usize;
            self.buffer.reserve_exact(span.min(BUFFER_SIZE));
        }
        if self.buffer.len() < total_bytes {
            self.buffer.resize(total_bytes, 0);
        }
        self.file
            .read_exact(&mut self.buffer[..total_bytes])
            .unwrap();
        self.buffer_len = total_bytes;
        self.buffer_start_offset = start_offset;
        self.file_position += total_bytes as u64;

        Some(())
    }

    fn decode_from_buffer(&mut self, page_index: usize, page: Page) {
        let header_start = (page.header_start - self.buffer_start_offset) as usize;
        let header_end = (page.header_end - self.buffer_start_offset) as usize;
        let body_start = (page.start - self.buffer_start_offset) as usize;
        let body_end = body_start + page.bytes as usize;
        let header = &self.buffer[header_start..header_end];
        let body = &self.buffer[body_start..body_end];
        let expected_len = page.values_count(Self::PER_PAGE, Self::SIZE_OF_T);
        // Our own data: a decode failure is corruption, never the end of the range.
        self.decoder
            .decode_into(page, header, body, expected_len, &mut self.decoded_values)
            .expect("compressed page decode failed");
        self.decoded_page_index = page_index;
    }

    fn ensure_page_buffered(&mut self, page_index: usize) -> Option<Page> {
        if page_index >= self.pages.len() {
            return None;
        }

        let page = self.pages.get(page_index)?;
        if self.buffer_len > 0 {
            let buffer_end_offset = self.buffer_start_offset + self.buffer_len as u64;
            if page.header_start >= self.buffer_start_offset && page.end() <= buffer_end_offset {
                return Some(page);
            }
        }

        self.refill_buffer(page_index)?;
        Some(page)
    }

    fn decode_page(&mut self, page_index: usize) -> Option<()> {
        let page = self.ensure_page_buffered(page_index)?;
        self.decode_from_buffer(page_index, page);
        Some(())
    }

    #[inline(always)]
    pub fn decoded_page(&mut self, page_index: usize) -> Option<&[T]> {
        self.ensure_page_decoded(page_index)?;
        Some(&self.decoded_values)
    }

    pub fn new<C: CachePolicy>(
        vec: &'a ReadWriteCompressedVec<I, T, S, C>,
        from: usize,
        to: usize,
    ) -> Self {
        Self::new_from_parts(vec.region(), vec.pages(), vec.stored_len(), from, to)
    }

    pub fn new_from_parts(
        region: &'a Region,
        pages: &'a Arc<RwLock<Pages>>,
        stored_len: usize,
        from: usize,
        to: usize,
    ) -> Self {
        // Pages before the region: the writer holds the pages while it rewrites the region.
        let pages = pages.read_recursive();
        let region_lock = region.read_lock();
        let region_start = region.meta().start() as u64;
        let file = region.open_db_read_only_file().expect("open file");
        let stored_len = stored_len.min(pages.stored_len(Self::PER_PAGE, Self::SIZE_OF_T));
        let from = from.min(stored_len);
        let to = to.min(stored_len);

        Self {
            file,
            file_position: 0,
            region_start,
            buffer: Vec::new(),
            buffer_len: 0,
            buffer_start_offset: 0,
            decoded_values: Vec::new(),
            decoder: PageDecoder::default(),
            decoded_page_index: Self::NO_PAGE,
            pages,
            index: from,
            end_index: to,
            _region_lock: region_lock,
            _marker: PhantomData,
        }
    }

    /// Reads remaining pages directly into the destination allocation.
    pub fn read_into(mut self, output: &mut Vec<T>) {
        if self.index >= self.end_index {
            return;
        }
        output.reserve(self.end_index - self.index);
        let start_page = self.index / Self::PER_PAGE;
        let end_page = (self.end_index - 1) / Self::PER_PAGE;
        let mut scratch = mem::take(&mut self.decoded_values);

        for page_index in start_page..=end_page {
            let page_start = page_index * Self::PER_PAGE;
            let page = self
                .ensure_page_buffered(page_index)
                .expect("compressed page should exist after bounds check");
            let header_start = (page.header_start - self.buffer_start_offset) as usize;
            let header_end = (page.header_end - self.buffer_start_offset) as usize;
            let body_start = (page.start - self.buffer_start_offset) as usize;
            let body_end = body_start + page.bytes as usize;
            let header = &self.buffer[header_start..header_end];
            let body = &self.buffer[body_start..body_end];
            let values_count = page.values_count(Self::PER_PAGE, Self::SIZE_OF_T);
            let local_from = self.index.saturating_sub(page_start);
            let local_to = (self.end_index - page_start).min(values_count);
            self.decoder
                .decode_range_append(
                    page,
                    header,
                    body,
                    values_count,
                    local_from..local_to,
                    &mut scratch,
                    output,
                )
                .expect("page decode failed in read_into_at");
        }
    }

    /// Lends the rest of the next page within the range with its first index, advancing past it.
    #[inline(always)]
    fn next_chunk(&mut self) -> Option<(usize, &[T])> {
        if self.index >= self.end_index {
            return None;
        }
        let page_index = self.index / Self::PER_PAGE;
        let page_start = page_index * Self::PER_PAGE;
        self.ensure_page_decoded(page_index)?;
        let from = self.index - page_start;
        let to = (self.end_index - page_start).min(self.decoded_values.len());
        if from >= to {
            return None;
        }
        let at = self.index;
        self.index = page_start + to;
        Some((at, &self.decoded_values[from..to]))
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
