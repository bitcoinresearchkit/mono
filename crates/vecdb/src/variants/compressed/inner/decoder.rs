use std::{marker::PhantomData, ops::Range};

use super::{CompressionStrategy, Page};
use crate::{Error, Result, VecValue};

pub struct PageDecoder<T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    /// Header start of the chunk `decoder` was reset for.
    chunk_start: u64,
    decoder: S::Decoder,
    marker: PhantomData<T>,
}

impl<T, S> Default for PageDecoder<T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    fn default() -> Self {
        Self {
            chunk_start: u64::MAX,
            decoder: S::Decoder::default(),
            marker: PhantomData,
        }
    }
}

impl<T, S> PageDecoder<T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    fn decoder(&mut self, page: Page, header: &[u8]) -> Result<&mut S::Decoder> {
        if self.chunk_start != page.header_start {
            // A failed reset leaves no chunk current.
            self.chunk_start = u64::MAX;
            S::reset_decoder(&mut self.decoder, header)?;
            self.chunk_start = page.header_start;
        }
        Ok(&mut self.decoder)
    }

    pub fn decode_into(
        &mut self,
        page: Page,
        header: &[u8],
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> Result<()> {
        if page.is_raw() {
            S::bytes_to_values_into(body, expected_len, dst)?;
        } else {
            S::decompress_page_into(self.decoder(page, header)?, body, expected_len, dst)?;
        }
        // Cursors read decoded pages unchecked: a strategy's length is checked once here.
        if dst.len() != expected_len {
            return Err(Error::DecompressionMismatch {
                expected_len,
                actual_len: dst.len(),
            });
        }
        Ok(())
    }

    /// Appends values `range` of one page: a raw page converts only those bytes; a compressed
    /// page decodes in place when the range starts it, else through `scratch`.
    #[allow(clippy::too_many_arguments)]
    pub fn decode_range_append(
        &mut self,
        page: Page,
        header: &[u8],
        body: &[u8],
        values_count: usize,
        range: Range<usize>,
        scratch: &mut Vec<T>,
        dst: &mut Vec<T>,
    ) -> Result<()> {
        if page.is_raw() {
            return Self::raw_range_append(body, range, dst);
        }
        let decoder = self.decoder(page, header)?;
        if range.start == 0 {
            let before = dst.len();
            S::decompress_page_append(decoder, body, values_count, dst)?;
            dst.truncate(before + range.end);
        } else {
            S::decompress_page_into(decoder, body, values_count, scratch)?;
            dst.extend_from_slice(&scratch[range]);
        }
        Ok(())
    }

    /// Appends values `range` of a raw page, converting only those bytes.
    pub fn raw_range_append(body: &[u8], range: Range<usize>, dst: &mut Vec<T>) -> Result<()> {
        let size = size_of::<T>();
        S::bytes_to_values_append(
            &body[range.start * size..range.end * size],
            range.len(),
            dst,
        )
    }
}
