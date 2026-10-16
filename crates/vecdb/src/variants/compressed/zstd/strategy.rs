use std::marker::PhantomData;

use zstd::{bulk::Decompressor, encode_all};

use super::super::inner::CompressionStrategy;
use crate::{BytesVecValue, EncodedChunk, Format, Result, ValueStrategy};

/// Zstd compression level (1-22). Level 3 provides a good balance
/// between compression ratio and speed for most workloads.
const ZSTD_COMPRESSION_LEVEL: i32 = 3;

/// Zstd compression strategy for high compression ratios.
#[derive(Debug, Clone, Copy)]
pub struct ZstdStrategy<T>(PhantomData<T>);

impl_bytes_value_strategy!(ZstdStrategy, BytesVecValue);

impl<T> ZstdStrategy<T>
where
    T: BytesVecValue,
{
    /// Decompresses one page into the decoder's byte scratch.
    fn decompress_bytes<'a>(
        (context, bytes): &'a mut <Self as CompressionStrategy<T>>::Decoder,
        body: &[u8],
        expected_len: usize,
    ) -> Result<&'a [u8]> {
        let context = match context {
            Some(context) => context,
            None => context.insert(Decompressor::new()?),
        };
        bytes.clear();
        // Room for the page; the caller checks the decoded length.
        bytes.reserve(expected_len * size_of::<T>());
        context.decompress_to_buffer(body, bytes)?;
        Ok(bytes)
    }
}

impl<T> CompressionStrategy<T> for ZstdStrategy<T>
where
    T: BytesVecValue,
{
    /// A decompression context and a byte scratch buffer, both created once.
    type Decoder = (Option<Decompressor<'static>>, Vec<u8>);

    const FORMAT: Format = Format::Zstd;

    const MAX_UNCOMPRESSED_CHUNK_SIZE: usize = 8 * 1024;

    fn compress_chunk(values: &[T], _values_per_page: usize) -> Result<EncodedChunk> {
        let bytes = Self::values_bytes(values);
        EncodedChunk::single_page(encode_all(&*bytes, ZSTD_COMPRESSION_LEVEL)?)
    }

    fn reset_decoder(_decoder: &mut Self::Decoder, _header: &[u8]) -> Result<()> {
        Ok(())
    }

    fn decompress_page_append(
        decoder: &mut Self::Decoder,
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> Result<()> {
        let bytes = Self::decompress_bytes(decoder, body, expected_len)?;
        Self::bytes_to_values_append(bytes, expected_len, dst)
    }
}
