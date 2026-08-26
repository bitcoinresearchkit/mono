use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    sync::Arc,
};

use parking_lot::{RwLock, RwLockReadGuard};
use rawdb::{Region, RegionMetadata};

use crate::{AnyStoredVec, BUFFER_SIZE, Pages, VecIndex, VecValue, likely, unlikely};

use super::super::inner::{
    CompressionStrategy, MAX_UNCOMPRESSED_PAGE_SIZE, Page, ReadWriteCompressedVec,
};

/// Buffered file I/O source for reading stored compressed data.
///
/// Uses dedicated file handle for sequential reads (better OS readahead than mmap).
/// Only sees stored (persisted) values. Consumed by fold/try_fold/for_each.
pub struct CompressedIoSource<'a, I, T, S> {
    file: File,
    file_position: u64,
    region_start: u64,
    buffer: Vec<u8>,
    buffer_len: usize,
    buffer_start_offset: u64,
    decoded_values: Vec<T>,
    decoded_page_index: usize,
    pages: RwLockReadGuard<'a, Pages>,
    index: usize,
    end_index: usize,
    _region_lock: RwLockReadGuard<'a, RegionMetadata>,
    _marker: std::marker::PhantomData<(I, T, S)>,
}

impl<'a, I, T, S> CompressedIoSource<'a, I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    const SIZE_OF_T: usize = size_of::<T>();
    const PER_PAGE: usize = MAX_UNCOMPRESSED_PAGE_SIZE / Self::SIZE_OF_T;
    const NO_PAGE: usize = usize::MAX;

    pub(crate) fn new(vec: &'a ReadWriteCompressedVec<I, T, S>, from: usize, to: usize) -> Self {
        Self::new_from_parts(vec.region(), vec.pages(), vec.stored_len(), from, to)
    }

    pub(crate) fn new_from_parts(
        region: &'a Region,
        pages: &'a Arc<RwLock<Pages>>,
        stored_len: usize,
        from: usize,
        to: usize,
    ) -> Self {
        let region_lock = region.meta();
        let region_start = region_lock.start() as u64;
        let file = region.open_db_read_only_file().expect("open file");
        let pages = pages.read();
        let from = from.min(stored_len);
        let to = to.min(stored_len);

        Self {
            file,
            file_position: 0,
            region_start,
            buffer: vec![0; BUFFER_SIZE],
            buffer_len: 0,
            buffer_start_offset: 0,
            decoded_values: Vec::with_capacity(Self::PER_PAGE),
            decoded_page_index: Self::NO_PAGE,
            pages,
            index: from,
            end_index: to,
            _region_lock: region_lock,
            _marker: std::marker::PhantomData,
        }
    }

    #[inline(always)]
    fn ensure_page_decoded(&mut self, page_index: usize) -> Option<()> {
        if unlikely(self.decoded_page_index != page_index) {
            self.decode_page(page_index)?;
        }
        Some(())
    }

    fn refill_buffer(&mut self, starting_page_index: usize) -> Option<()> {
        let start_page = self.pages.get(starting_page_index)?;
        let start_offset = start_page.start;

        let last_needed_page = if self.end_index == 0 {
            0
        } else {
            (self.end_index - 1) / Self::PER_PAGE
        };
        let max_page = last_needed_page.min(self.pages.len().saturating_sub(1));

        let mut total_bytes = 0usize;
        for i in starting_page_index..=max_page {
            let page = self.pages.get(i)?;
            let page_bytes = page.bytes as usize;
            if total_bytes + page_bytes > BUFFER_SIZE {
                break;
            }
            total_bytes += page_bytes;
        }

        if total_bytes == 0 {
            return None;
        }

        let absolute_offset = self.region_start + start_offset;
        if self.file_position != absolute_offset {
            self.file_position = absolute_offset;
            self.file.seek(SeekFrom::Start(absolute_offset)).unwrap();
        }

        self.file
            .read_exact(&mut self.buffer[..total_bytes])
            .unwrap();
        self.buffer_len = total_bytes;
        self.buffer_start_offset = start_offset;
        self.file_position += total_bytes as u64;

        Some(())
    }

    fn decode_from_buffer(&mut self, page_index: usize, page: Page) -> Option<()> {
        let in_buffer_offset = (page.start - self.buffer_start_offset) as usize;
        let data = &self.buffer[in_buffer_offset..in_buffer_offset + page.bytes as usize];

        S::decode_page_into(data, &page, &mut self.decoded_values).ok()?;
        self.decoded_page_index = page_index;

        Some(())
    }

    fn ensure_page_buffered(&mut self, page_index: usize) -> Option<Page> {
        if page_index >= self.pages.len() {
            return None;
        }

        let page = *self.pages.get(page_index)?;
        if self.buffer_len > 0 {
            let buffer_end_offset = self.buffer_start_offset + self.buffer_len as u64;
            if page.start >= self.buffer_start_offset && page.end() <= buffer_end_offset {
                return Some(page);
            }
        }

        self.refill_buffer(page_index)?;
        Some(page)
    }

    fn decode_page(&mut self, page_index: usize) -> Option<()> {
        let page = self.ensure_page_buffered(page_index)?;
        self.decode_from_buffer(page_index, page)
    }

    #[inline(always)]
    pub fn decoded_page(&mut self, page_index: usize) -> Option<&[T]> {
        self.ensure_page_decoded(page_index)?;
        Some(&self.decoded_values)
    }

    /// Reads remaining pages directly into the destination allocation.
    pub(crate) fn read_into(mut self, output: &mut Vec<T>) {
        output.reserve(self.end_index - self.index);
        let start_page = self.index / Self::PER_PAGE;
        let end_page = (self.end_index - 1) / Self::PER_PAGE;
        let mut page_buf = std::mem::take(&mut self.decoded_values);

        for page_index in start_page..=end_page {
            let page_start = page_index * Self::PER_PAGE;
            let page = self
                .ensure_page_buffered(page_index)
                .expect("compressed page should exist after bounds check");
            let local = (page.start - self.buffer_start_offset) as usize;
            let data = &self.buffer[local..local + page.bytes as usize];
            let values_count = page.values_count() as usize;
            let local_from = self.index.saturating_sub(page_start);
            let local_to = (self.end_index - page_start).min(values_count);

            if !page.is_raw() && likely(local_from == 0) {
                let before = output.len();
                S::decompress_append(data, values_count, output)
                    .expect("decompression failed in read_into_at");
                output.truncate(before + local_to);
            } else {
                S::decode_page_into(data, &page, &mut page_buf)
                    .expect("page decode failed in read_into_at");
                output.extend_from_slice(&page_buf[local_from..local_to]);
            }
        }
    }

    /// Fold all remaining elements — tight pointer loop per page so LLVM can vectorize.
    #[inline(always)]
    pub(crate) fn fold<B, F: FnMut(B, T) -> B>(mut self, init: B, mut f: F) -> B {
        let per_page = Self::PER_PAGE;
        let end_index = self.end_index;
        let mut page_index = self.index / per_page;
        let mut page_start = page_index * per_page;
        let mut in_page_offset = self.index - page_start;
        let mut accum = init;
        while self.index < end_index {
            if self.ensure_page_decoded(page_index).is_none() {
                break;
            }
            let page_end = (end_index - page_start).min(self.decoded_values.len());
            let ptr = self.decoded_values.as_ptr();
            let mut index = in_page_offset;
            while index < page_end {
                // Clone rather than move: decoded_values still owns and drops every value.
                accum = f(accum, unsafe { (&*ptr.add(index)).clone() });
                index += 1;
            }
            self.index = page_start + page_end;
            page_index += 1;
            page_start += per_page;
            in_page_offset = 0;
        }
        accum
    }

    /// Fallible fold with early exit on error.
    #[inline(always)]
    pub(crate) fn try_fold<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        mut self,
        init: B,
        mut f: F,
    ) -> std::result::Result<B, E> {
        let per_page = Self::PER_PAGE;
        let end_index = self.end_index;
        let mut page_index = self.index / per_page;
        let mut page_start = page_index * per_page;
        let mut in_page_offset = self.index - page_start;
        let mut accum = init;
        while self.index < end_index {
            if self.ensure_page_decoded(page_index).is_none() {
                break;
            }
            let page_end = (end_index - page_start).min(self.decoded_values.len());
            for value in &self.decoded_values[in_page_offset..page_end] {
                accum = f(accum, value.clone())?;
            }
            self.index = page_start + page_end;
            page_index += 1;
            page_start += per_page;
            in_page_offset = 0;
        }
        Ok(accum)
    }
}

#[cfg(all(test, feature = "pco"))]
mod tests {
    use tempfile::tempdir;

    use crate::{AnyStoredVec, Database, ImportableVec, PcoVec, Version, WritableVec};

    use super::CompressedIoSource;

    #[test]
    fn read_into_handles_full_and_partial_pages() {
        let temp = tempdir().unwrap();
        let db = Database::open(temp.path()).unwrap();
        let mut vec: PcoVec<usize, u64> =
            PcoVec::forced_import(&db, "values", Version::ONE).unwrap();
        let per_page = 16 * 1024 / size_of::<u64>();
        let values: Vec<_> = (0..per_page * 3 + 137)
            .map(|index| index as u64 * 17 + index as u64 % 11)
            .collect();
        for &value in &values {
            vec.push(value);
        }
        vec.write().unwrap();

        for (from, to) in [
            (0, values.len()),
            (17, per_page + 9),
            (per_page, per_page * 3),
            (per_page * 2 + 31, values.len() - 7),
        ] {
            let mut output = vec![u64::MAX];
            CompressedIoSource::new(&vec, from, to).read_into(&mut output);
            assert_eq!(&output[1..], &values[from..to]);
        }
    }
}
