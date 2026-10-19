use std::{result::Result, sync::Arc};

use parking_lot::{RwLock, RwLockReadGuard};

use super::super::inner::{COMPRESSED_PAGE_SIZE, CompressionStrategy, Page, PageDecoder};
use crate::{Pages, Result as CrateResult, VecValue, unlikely};

/// Where a [`CompressedSource`] gets its page bytes: the mapping or a buffered file.
pub trait PageBytes {
    /// The header and body bytes of `page`, page `page_index` of `pages`. `last_page` bounds
    /// any read-ahead: the last page the source reads.
    fn page(
        &mut self,
        pages: &Pages,
        page_index: usize,
        page: Page,
        last_page: usize,
    ) -> Option<(&[u8], &[u8])>;
}

/// Read-only source over a bounded range of a compressed vector's stored (persisted) values.
///
/// Pages are decoded lazily, one at a time through one decoder, and consumed by
/// `try_fold`/`for_each_chunk`/`read_into` or lent to cursors.
pub struct CompressedSource<'a, T, S, B>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    bytes: B,
    pages: RwLockReadGuard<'a, Pages>,
    decoded: Vec<T>,
    decoder: PageDecoder<T, S>,
    decoded_page: usize,
    pos: usize,
    end: usize,
}

impl<'a, T, S, B> CompressedSource<'a, T, S, B>
where
    T: VecValue,
    S: CompressionStrategy<T>,
    B: PageBytes,
{
    const SIZE_OF_T: usize = size_of::<T>();
    const PER_PAGE: usize = COMPRESSED_PAGE_SIZE / Self::SIZE_OF_T;
    const NO_PAGE: usize = usize::MAX;

    /// Takes the page table before `bytes` takes the region: the writer holds the pages while it
    /// rewrites the region. The range is bounded by the published page table.
    pub(super) fn with_bytes(
        pages: &'a Arc<RwLock<Pages>>,
        bytes: impl FnOnce() -> B,
        stored_len: usize,
        from: usize,
        to: usize,
    ) -> Self {
        let pages = pages.read_recursive();
        let bytes = bytes();
        let stored_len = stored_len.min(pages.stored_len(Self::PER_PAGE, Self::SIZE_OF_T));
        let end = to.min(stored_len);
        Self {
            bytes,
            pages,
            decoded: Vec::new(),
            decoder: PageDecoder::default(),
            decoded_page: Self::NO_PAGE,
            pos: from.min(end),
            end,
        }
    }

    #[inline(always)]
    fn last_page(&self) -> usize {
        self.end.saturating_sub(1) / Self::PER_PAGE
    }

    /// The whole decoded page `page_index`, decoding it unless it is the current one.
    #[inline(always)]
    pub fn decoded_page(&mut self, page_index: usize) -> Option<&[T]> {
        if unlikely(self.decoded_page != page_index) {
            self.decode_page(page_index)?;
        }
        Some(&self.decoded)
    }

    fn decode_page(&mut self, page_index: usize) -> Option<()> {
        let page = self.pages.get(page_index)?;
        let last_page = self.last_page();
        let (header, body) = self.bytes.page(&self.pages, page_index, page, last_page)?;
        let values_count = page.values_count(Self::PER_PAGE, Self::SIZE_OF_T);
        // Our own data: a decode failure is corruption, never the end of the range.
        self.decoder
            .decode_into(page, header, body, values_count, &mut self.decoded)
            .expect("compressed page decode failed");
        self.decoded_page = page_index;
        Some(())
    }

    /// Lends the rest of the next page within the range with its first index, advancing past it.
    #[inline(always)]
    fn next_chunk(&mut self) -> Option<(usize, &[T])> {
        if self.pos >= self.end {
            return None;
        }
        let page_index = self.pos / Self::PER_PAGE;
        let page_start = page_index * Self::PER_PAGE;
        let from = self.pos - page_start;
        let at = self.pos;
        let page = self.pages.get(page_index)?;
        let to = (self.end - page_start).min(page.values_count(Self::PER_PAGE, Self::SIZE_OF_T));
        // `pos < end`, and `end` is within the published page table.
        debug_assert!(from < to);
        let values = if page.raw {
            // The raw tail page converts only the requested values, not the whole page.
            let last_page = self.last_page();
            let (_, body) = self.bytes.page(&self.pages, page_index, page, last_page)?;
            self.decoded.clear();
            self.decoded_page = Self::NO_PAGE;
            PageDecoder::<T, S>::raw_range_append(body, from..to, &mut self.decoded)
                .expect("raw page conversion failed");
            0..to - from
        } else {
            self.decoded_page(page_index)?;
            from..to
        };
        self.pos = page_start + to;
        Some((at, &self.decoded[values]))
    }

    /// Visits the range one decoded page at a time.
    pub fn for_each_chunk(mut self, f: &mut dyn FnMut(usize, &[T])) {
        while let Some((at, values)) = self.next_chunk() {
            f(at, values);
        }
    }

    #[inline(always)]
    pub fn try_fold<Acc, E, F: FnMut(Acc, T) -> Result<Acc, E>>(
        mut self,
        init: Acc,
        mut f: F,
    ) -> Result<Acc, E> {
        let mut accum = init;
        while let Some((_, values)) = self.next_chunk() {
            accum = values.iter().cloned().try_fold(accum, &mut f)?;
        }
        Ok(accum)
    }

    /// Appends the range, decoding each page directly into `out` when the range starts it.
    pub fn read_into(mut self, out: &mut Vec<T>) -> CrateResult<()> {
        if self.pos >= self.end {
            return Ok(());
        }
        out.reserve(self.end - self.pos);
        let last_page = self.last_page();
        let mut scratch = Vec::new();
        for page_index in self.pos / Self::PER_PAGE..=last_page {
            let page_start = page_index * Self::PER_PAGE;
            let page = self.pages.get(page_index).expect("missing compressed page");
            let (header, body) = self
                .bytes
                .page(&self.pages, page_index, page, last_page)
                .expect("unreadable compressed page");
            let values_count = page.values_count(Self::PER_PAGE, Self::SIZE_OF_T);
            let local_from = self.pos.saturating_sub(page_start);
            let local_to = (self.end - page_start).min(values_count);
            self.decoder.decode_range_append(
                page,
                header,
                body,
                values_count,
                local_from..local_to,
                &mut scratch,
                out,
            )?;
        }
        Ok(())
    }
}
