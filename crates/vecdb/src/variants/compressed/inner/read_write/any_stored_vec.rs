use std::{mem, ops::Range, path::PathBuf};

use rawdb::{Database, Region};
use rayon::prelude::*;

use crate::{AnyStoredVec, AnyVec, Error, Header, Stamp, VecIndex, VecValue, WritableVec};

use super::super::{CompressionStrategy, PAGES_PER_BLOCK};
use super::ReadWriteCompressedVec;

impl<I, T, S> ReadWriteCompressedVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    fn encode_pages(
        &self,
        values: &[T],
        starting_page_index: usize,
    ) -> crate::Result<(Vec<u8>, Vec<(Option<u32>, u32, bool)>)> {
        let full_pages = values.len() / Self::PER_PAGE;
        let mut page_offset = 0;
        let mut ranges = Vec::<Range<usize>>::new();
        while page_offset < full_pages {
            let page_index = starting_page_index + page_offset;
            let block_remaining = PAGES_PER_BLOCK - page_index % PAGES_PER_BLOCK;
            let chunk_pages = self
                .pages_per_chunk
                .min(block_remaining)
                .min(full_pages - page_offset);
            let from = page_offset * Self::PER_PAGE;
            let to = (page_offset + chunk_pages) * Self::PER_PAGE;
            ranges.push(from..to);
            page_offset += chunk_pages;
        }

        let chunks = ranges
            .par_iter()
            .map(|range| S::compress_chunk(&values[range.clone()], Self::PER_PAGE))
            .collect::<crate::Result<Vec<_>>>()?;
        let mut bytes = Vec::new();
        let mut layouts = Vec::with_capacity(values.len().div_ceil(Self::PER_PAGE));
        for (chunk, range) in chunks.into_iter().zip(ranges) {
            let expected_pages = range.len() / Self::PER_PAGE;
            if chunk.page_count() != expected_pages {
                return Err(Error::InvalidArgument(
                    "compression strategy returned the wrong page count",
                ));
            }
            let (chunk_bytes, header_len, page_ends) = chunk.into_parts();
            let mut previous_end = header_len;
            for (index, end) in page_ends.into_iter().enumerate() {
                let body_bytes = end.checked_sub(previous_end).ok_or(Error::Underflow)?;
                layouts.push(((index == 0).then_some(header_len), body_bytes, false));
                previous_end = end;
            }
            bytes.extend(chunk_bytes);
        }

        let raw_from = full_pages * Self::PER_PAGE;
        if raw_from < values.len() {
            let raw = S::values_to_bytes(&values[raw_from..]);
            let raw_len = u32::try_from(raw.len()).map_err(|_| Error::Overflow)?;
            bytes.extend(raw);
            layouts.push((Some(0), raw_len, true));
        }

        Ok((bytes, layouts))
    }
}

impl<I, T, S> AnyStoredVec for ReadWriteCompressedVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    #[inline]
    fn db_path(&self) -> PathBuf {
        self.base.db_path()
    }

    #[inline]
    fn region(&self) -> &Region {
        self.base.region()
    }

    #[inline]
    fn header(&self) -> &Header {
        self.base.header()
    }

    #[inline]
    fn mut_header(&mut self) -> &mut Header {
        self.base.mut_header()
    }

    #[inline]
    fn saved_stamped_changes(&self) -> u16 {
        self.base.saved_stamped_changes()
    }

    #[inline]
    fn stored_len(&self) -> usize {
        self.base.stored_len()
    }

    #[inline]
    fn real_stored_len(&self) -> usize {
        self.pages
            .read()
            .stored_len(Self::PER_PAGE, Self::SIZE_OF_T)
    }

    fn write(&mut self) -> crate::Result<bool> {
        self.base.write_header_if_needed()?;

        let stored_len = self.stored_len();
        let pushed_len = self.base.pushed().len();

        let (truncate_at, starting_page_index, partial_page) = {
            let pages = self.pages.read();

            let real_stored_len = pages.stored_len(Self::PER_PAGE, Self::SIZE_OF_T);
            if stored_len > real_stored_len {
                return Err(Error::CorruptedRegion {
                    name: self.name().to_string(),
                    region_len: real_stored_len,
                });
            }

            if pushed_len == 0 && stored_len == real_stored_len {
                return Ok(false);
            }

            let starting_page_index = Self::index_to_page_index(stored_len);
            if starting_page_index > pages.len() {
                return Err(Error::CorruptedRegion {
                    name: self.name().to_string(),
                    region_len: pages.len(),
                });
            }

            if starting_page_index < pages.len() {
                let partial_len = stored_len % Self::PER_PAGE;
                let page = pages
                    .get(starting_page_index)
                    .ok_or(Error::ExpectVecToHaveIndex)?;
                let rewrite_page_index = page.chunk_start_page;
                let rewrite_page = pages
                    .get(rewrite_page_index)
                    .ok_or(Error::ExpectVecToHaveIndex)?;
                (
                    rewrite_page.header_start,
                    rewrite_page_index,
                    if partial_len != 0 {
                        Some((page, partial_len, starting_page_index))
                    } else {
                        None
                    },
                )
            } else {
                (pages.next_start(), starting_page_index, None)
            }
        };
        // Pages lock released — decompression happens without blocking readers

        // Fast path: append to existing raw page without reading it back.
        // When the last page is raw, not truncated, and won't overflow, just
        // write the new pushed bytes at the end of the existing page data.
        if let Some((page, partial_len, page_index)) = partial_page
            && page.is_raw()
            && starting_page_index == page_index
            && partial_len == page.values_count(Self::PER_PAGE, Self::SIZE_OF_T)
            && partial_len + pushed_len < Self::PER_PAGE
        {
            let taken = mem::take(self.base.mut_pushed());
            let raw = S::values_to_bytes(&taken);
            let append_at = page.end() as usize;
            self.region().truncate_write(append_at, &raw)?;

            let mut pages = self.pages.write();
            pages.truncate(starting_page_index);
            let total_bytes = page
                .bytes
                .checked_add(u32::try_from(raw.len()).map_err(|_| Error::Overflow)?)
                .ok_or(Error::Overflow)?;
            pages.push_raw(starting_page_index, total_bytes)?;
            self.base.update_stored_len(stored_len + pushed_len);
            pages.flush()?;
            return Ok(true);
        }

        // Rebuild only the bounded chunk that contains the truncation point.
        let rewrite_from = Self::page_index_to_index(starting_page_index);
        let mut values = if rewrite_from < stored_len {
            self.collect_stored_range(rewrite_from, stored_len)?
        } else {
            vec![]
        };

        // Encode pages with no locks held. Full pages compress; the last
        // partial page is stored raw (avoids recompression on every write).
        let taken = mem::take(self.base.mut_pushed());
        if values.is_empty() {
            values = taken;
        } else {
            values.extend(taken);
        }

        let (buf, page_layouts) = self.encode_pages(&values, starting_page_index)?;

        // Write the region before re-taking the pages lock to avoid deadlock.
        self.region().truncate_write(truncate_at as usize, &buf)?;

        let mut pages = self.pages.write();
        pages.truncate(starting_page_index);

        for (offset, &(header_bytes, body_bytes, is_raw)) in page_layouts.iter().enumerate() {
            let page_index = starting_page_index + offset;
            if is_raw {
                pages.push_raw(page_index, body_bytes)?;
            } else {
                pages.push_compressed(page_index, header_bytes, body_bytes)?;
            }
        }

        self.base.update_stored_len(stored_len + pushed_len);
        pages.flush()?;

        Ok(true)
    }

    #[inline]
    fn serialize_changes(&self) -> crate::Result<Vec<u8>> {
        self.serialize_compressed_changes()
    }

    #[inline]
    fn db(&self) -> Database {
        self.base.db()
    }

    fn any_stamped_write_with_changes(&mut self, stamp: Stamp) -> crate::Result<()> {
        <Self as WritableVec<I, T>>::stamped_write_with_changes(self, stamp)
    }

    fn any_save_rollback_state(&mut self) {
        <Self as WritableVec<I, T>>::save_rollback_state(self)
    }

    fn remove(self) -> crate::Result<()> {
        Self::remove(self)
    }

    fn any_truncate_if_needed_at(&mut self, index: usize) -> crate::Result<()> {
        <Self as WritableVec<I, T>>::truncate_if_needed_at(self, index)
    }

    fn any_reset(&mut self) -> crate::Result<()> {
        <Self as WritableVec<I, T>>::reset(self)
    }
}
