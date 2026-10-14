use std::marker::PhantomData;

use super::{CompressionStrategy, Page};
use crate::{Result, VecValue};

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
            S::bytes_to_values_into(body, expected_len, dst)
        } else {
            S::decompress_page_into(self.decoder(page, header)?, body, expected_len, dst)
        }
    }

    pub fn decode_append(
        &mut self,
        page: Page,
        header: &[u8],
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> Result<()> {
        if page.is_raw() {
            S::bytes_to_values_append(body, expected_len, dst)
        } else {
            S::decompress_page_append(self.decoder(page, header)?, body, expected_len, dst)
        }
    }
}
