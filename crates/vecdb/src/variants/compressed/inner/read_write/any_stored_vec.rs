use std::{mem, path::PathBuf};

use rawdb::{Database, Region};
use rayon::prelude::*;

use crate::{AnyStoredVec, AnyVec, Error, Header, Stamp, VecIndex, VecValue, WritableVec};

use super::super::{CompressionStrategy, Page};
use super::ReadWriteCompressedVec;

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
        self.pages.read().stored_len(Self::PER_PAGE)
    }

    fn write(&mut self) -> crate::Result<bool> {
        self.base.write_header_if_needed()?;

        let stored_len = self.stored_len();
        let pushed_len = self.base.pushed().len();

        let (truncate_at, starting_page_index, partial_page) = {
            let pages = self.pages.read();

            let real_stored_len = pages.stored_len(Self::PER_PAGE);
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
                let page = *pages
                    .get(starting_page_index)
                    .ok_or(Error::ExpectVecToHaveIndex)?;
                (
                    page.start,
                    starting_page_index,
                    if partial_len != 0 {
                        Some((page, partial_len))
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
        if let Some((page, partial_len)) = partial_page
            && page.is_raw()
            && partial_len == page.values_count() as usize
            && partial_len + pushed_len < Self::PER_PAGE
        {
            let taken = mem::take(self.base.mut_pushed());
            let raw = S::values_to_bytes(&taken);
            let append_at = page.end() as usize;
            self.region().truncate_write(append_at, &raw)?;

            let mut pages = self.pages.write();
            pages.truncate(starting_page_index);
            pages.checked_push(
                starting_page_index,
                Page::raw(
                    page.start,
                    page.bytes + raw.len() as u32,
                    (partial_len + pushed_len) as u32,
                ),
            )?;
            self.base.update_stored_len(stored_len + pushed_len);
            pages.flush()?;
            return Ok(true);
        }

        // Decompress the partial page outside the pages lock.
        let mut values = if let Some((page, partial_len)) = partial_page {
            let reader = self.create_reader();
            let data = reader.unchecked_read(page.start as usize, page.bytes as usize);
            let mut page_values = S::decode_page(data, &page)?;
            page_values.truncate(partial_len);
            page_values
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

        let num_pages = values.len().div_ceil(Self::PER_PAGE);
        let encode_page = |chunk: &[T]| {
            let (encoded, is_raw) = if chunk.len() == Self::PER_PAGE {
                (Self::compress_page(chunk)?, false)
            } else {
                (S::values_to_bytes(chunk), true)
            };
            Ok::<_, Error>((encoded, chunk.len(), is_raw))
        };
        let encoded_pages = if num_pages == 1 {
            vec![encode_page(&values)?]
        } else {
            values
                .par_chunks(Self::PER_PAGE)
                .map(encode_page)
                .collect::<crate::Result<Vec<_>>>()?
        };
        let encoded_len = encoded_pages.iter().map(|(page, _, _)| page.len()).sum();
        let mut buf = Vec::with_capacity(encoded_len);
        let mut page_sizes = Vec::with_capacity(num_pages);
        for (encoded, values_len, is_raw) in encoded_pages {
            page_sizes.push((encoded.len(), values_len, is_raw));
            buf.extend_from_slice(&encoded);
        }

        // Write the region before re-taking the pages lock to avoid deadlock.
        self.region().truncate_write(truncate_at as usize, &buf)?;

        let mut pages = self.pages.write();
        pages.truncate(starting_page_index);

        for (i, &(byte_len, values_len, is_raw)) in page_sizes.iter().enumerate() {
            let start = pages.next_start();
            let page = if is_raw {
                Page::raw(start, byte_len as u32, values_len as u32)
            } else {
                Page::compressed(start, byte_len as u32, values_len as u32)
            };
            pages.checked_push(starting_page_index + i, page)?;
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
