use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    sync::Arc,
};

use parking_lot::{RwLock, RwLockReadGuard};
use rawdb::Region;

use super::{
    super::inner::{CompressionStrategy, Page},
    CompressedSource, PageBytes,
};
use crate::{BUFFER_SIZE, Pages, VecValue};

/// Page bytes read through a dedicated file handle in buffered spans of whole pages (better OS
/// readahead than mmap for cold sequential scans).
pub struct Buffered<'a> {
    file: File,
    file_position: u64,
    region_start: u64,
    buffer: Vec<u8>,
    buffer_len: usize,
    buffer_start_offset: u64,
    _region_lock: RwLockReadGuard<'a, ()>,
}

impl Buffered<'_> {
    /// Reads whole pages from `starting_page_index`, up to `last_page` and [`BUFFER_SIZE`].
    fn refill(
        &mut self,
        pages: &Pages,
        starting_page_index: usize,
        last_page: usize,
    ) -> Option<()> {
        let start_offset = pages.get(starting_page_index)?.header_start;
        let max_page = last_page.min(pages.len().saturating_sub(1));

        let mut end_offset = start_offset;
        for i in starting_page_index..=max_page {
            let candidate_end = pages.get(i)?.end();
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
            let span = pages.get(max_page)?.end().checked_sub(start_offset)? as usize;
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
}

impl PageBytes for Buffered<'_> {
    fn page(
        &mut self,
        pages: &Pages,
        page_index: usize,
        page: Page,
        last_page: usize,
    ) -> Option<(&[u8], &[u8])> {
        let buffered = self.buffer_len > 0
            && page.header_start >= self.buffer_start_offset
            && page.end() <= self.buffer_start_offset + self.buffer_len as u64;
        if !buffered {
            self.refill(pages, page_index, last_page)?;
        }
        let header_start = (page.header_start - self.buffer_start_offset) as usize;
        let header_end = (page.header_end - self.buffer_start_offset) as usize;
        let body_start = (page.start - self.buffer_start_offset) as usize;
        let body_end = body_start + page.bytes as usize;
        Some((
            &self.buffer[header_start..header_end],
            &self.buffer[body_start..body_end],
        ))
    }
}

/// Buffered file I/O [`CompressedSource`].
pub type CompressedIoSource<'a, I, T, S> = CompressedSource<'a, I, T, S, Buffered<'a>>;

impl<'a, I, T, S> CompressedIoSource<'a, I, T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    pub fn new_from_parts(
        region: &'a Region,
        pages: &'a Arc<RwLock<Pages>>,
        stored_len: usize,
        from: usize,
        to: usize,
    ) -> Self {
        Self::with_bytes(
            pages,
            || Buffered {
                _region_lock: region.read_lock(),
                region_start: region.meta().start() as u64,
                file: region.open_db_read_only_file().expect("open file"),
                file_position: 0,
                buffer: Vec::new(),
                buffer_len: 0,
                buffer_start_offset: 0,
            },
            stored_len,
            from,
            to,
        )
    }
}
