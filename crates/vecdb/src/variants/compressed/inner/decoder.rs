use std::marker::PhantomData;

use crate::VecValue;

use super::{CompressionStrategy, Page};

pub struct PageDecoder<T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    chunk_start: u64,
    decoder: Option<S::Decoder>,
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
            decoder: None,
            marker: PhantomData,
        }
    }
}

impl<T, S> PageDecoder<T, S>
where
    T: VecValue,
    S: CompressionStrategy<T>,
{
    fn decoder(&mut self, page: Page, header: &[u8]) -> crate::Result<&mut S::Decoder> {
        if self.chunk_start != page.header_start {
            self.decoder = Some(S::decoder(header)?);
            self.chunk_start = page.header_start;
        }
        Ok(self.decoder.as_mut().expect("compressed page decoder"))
    }

    pub fn decode_into(
        &mut self,
        page: Page,
        header: &[u8],
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> crate::Result<()> {
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
    ) -> crate::Result<()> {
        if page.is_raw() {
            let mut values = Vec::with_capacity(expected_len);
            S::bytes_to_values_into(body, expected_len, &mut values)?;
            dst.extend(values);
            Ok(())
        } else {
            S::decompress_page_append(self.decoder(page, header)?, body, expected_len, dst)
        }
    }
}
