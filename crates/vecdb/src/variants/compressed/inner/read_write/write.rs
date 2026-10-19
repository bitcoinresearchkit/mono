use std::{mem, ops::Range};

use rayon::prelude::*;

use super::{
    super::{CompressionStrategy, PAGES_PER_BLOCK},
    ReadWriteCompressedVec,
};
use crate::{
    AnyStoredVec, AnyVec, Error, Result, VecIndex, VecValue, WritableVec, cache::CachePolicy,
};

impl<I, T, S, C: CachePolicy> ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    fn encode_pages(
        &self,
        values: &[T],
        starting_page_index: usize,
    ) -> Result<(Vec<u8>, Vec<(Option<u32>, u32, bool)>)> {
        let full_pages = values.len() / Self::PER_PAGE;
        let mut page_offset = 0;
        let mut ranges = Vec::<Range<usize>>::new();
        while page_offset < full_pages {
            let page_index = starting_page_index + page_offset;
            let block_remaining = PAGES_PER_BLOCK - page_index % PAGES_PER_BLOCK;
            let chunk_pages = Self::PAGES_PER_CHUNK
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
            .collect::<Result<Vec<_>>>()?;
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
            // `EncodedChunk` ends rise strictly past the header.
            for (index, end) in page_ends.into_iter().enumerate() {
                let body_bytes = end - previous_end;
                layouts.push(((index == 0).then_some(header_len), body_bytes, false));
                previous_end = end;
            }
            bytes.extend(chunk_bytes);
        }

        let raw_from = full_pages * Self::PER_PAGE;
        if raw_from < values.len() {
            // Less than a page: under `COMPRESSED_PAGE_SIZE` bytes.
            let raw = S::values_bytes(&values[raw_from..]);
            bytes.extend_from_slice(&raw);
            layouts.push((Some(0), raw.len() as u32, true));
        }

        Ok((bytes, layouts))
    }

    /// Rows on disk.
    #[inline]
    pub(super) fn written_len(&self) -> usize {
        self.pages
            .read()
            .stored_len(Self::PER_PAGE, Self::SIZE_OF_T)
    }

    /// Reads back and encodes outside the cache gate, so readers wait only for publication.
    pub(super) fn write_pushed(&mut self) -> Result<bool> {
        let stored_len = self.stored_len();
        let pushed_len = self.base.pushed().len();

        let (truncate_at, starting_page_index, partial_page) = {
            let pages = self.pages.read_recursive();

            let real_stored_len = pages.stored_len(Self::PER_PAGE, Self::SIZE_OF_T);
            if stored_len > real_stored_len {
                return Err(Error::CorruptedRegion {
                    name: self.name().to_string(),
                    region_len: real_stored_len,
                });
            }

            if pushed_len == 0 && stored_len == real_stored_len {
                return self.base.write_header_if_needed();
            }

            let starting_page_index = stored_len / Self::PER_PAGE;
            if starting_page_index > pages.len() {
                return Err(Error::CorruptedRegion {
                    name: self.name().to_string(),
                    region_len: pages.len(),
                });
            }

            if starting_page_index < pages.len() {
                let partial_len = stored_len % Self::PER_PAGE;
                let page = pages.get(starting_page_index).ok_or(Error::MissingPage)?;
                let rewrite_page_index = page.chunk_start_page;
                let rewrite_page = pages.get(rewrite_page_index).ok_or(Error::MissingPage)?;
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
        // Release the pages lock before accessing the data region.

        // Fast path: append to existing raw page without reading it back.
        if let Some((page, partial_len, page_index)) = partial_page
            && page.raw
            && starting_page_index == page_index
            && partial_len == page.values_count(Self::PER_PAGE, Self::SIZE_OF_T)
            && partial_len + pushed_len < Self::PER_PAGE
        {
            let taken = mem::take(self.base.mut_pushed());
            let raw = S::values_bytes(&taken);
            let append_at = page.end() as usize;
            // The grown page stays under a page: under `COMPRESSED_PAGE_SIZE` bytes.
            let total_bytes = page.bytes + raw.len() as u32;
            let result = self.with_cache_update(stored_len, |this| {
                this.publish(append_at, &raw, stored_len + pushed_len, |pages| {
                    pages.grow_last_raw(total_bytes)
                })?;
                if let Some(cache) = C::cache(&this.cache) {
                    cache.extend_tail(stored_len, &taken);
                }
                Ok(true)
            });
            drop(raw);
            self.base.recycle_pushed(taken);
            return result;
        }

        // Rebuild only the bounded chunk that contains the truncation point.
        let rewrite_from = starting_page_index * Self::PER_PAGE;
        let mut values = if rewrite_from < stored_len {
            self.collect_stored_range(rewrite_from, stored_len)?
        } else {
            vec![]
        };

        // Full pages compress; the last partial page is stored raw (avoids recompression on
        // every write).
        let taken = mem::take(self.base.mut_pushed());
        if values.is_empty() {
            values = taken;
        } else {
            values.extend(taken);
        }

        let (buf, page_layouts) = self.encode_pages(&values, starting_page_index)?;

        self.with_cache_update(stored_len, |this| {
            this.publish(
                truncate_at as usize,
                &buf,
                stored_len + pushed_len,
                |pages| {
                    pages.truncate(starting_page_index);
                    for &(header_bytes, body_bytes, is_raw) in &page_layouts {
                        if is_raw {
                            pages.push_raw(body_bytes)?;
                        } else {
                            pages.push_compressed(header_bytes, body_bytes)?;
                        }
                    }
                    Ok(())
                },
            )?;

            if let Some(cache) = C::cache(&this.cache) {
                cache.extend_tail(stored_len, &values[stored_len - rewrite_from..]);
            }

            Ok(true)
        })
    }
}
