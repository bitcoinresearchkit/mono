use std::marker::PhantomData;

use zstd::{decode_all, encode_all};

use crate::{EncodedChunk, impl_bytes_value_strategy};

use super::{super::inner::CompressionStrategy, value::ZstdVecValue};

/// Zstd compression level (1-22). Level 3 provides a good balance
/// between compression ratio and speed for most workloads.
const ZSTD_COMPRESSION_LEVEL: i32 = 3;

/// Zstd compression strategy for high compression ratios.
#[derive(Debug, Clone, Copy)]
pub struct ZstdStrategy<T>(PhantomData<T>);

impl_bytes_value_strategy!(ZstdStrategy, ZstdVecValue);

impl<T> CompressionStrategy<T> for ZstdStrategy<T>
where
    T: ZstdVecValue,
{
    type Decoder = ();

    const MAX_UNCOMPRESSED_CHUNK_SIZE: usize = 8 * 1024;

    fn compress_chunk(values: &[T], _values_per_page: usize) -> crate::Result<EncodedChunk> {
        let bytes = Self::values_to_bytes(values);
        EncodedChunk::single_page(encode_all(bytes.as_slice(), ZSTD_COMPRESSION_LEVEL)?)
    }

    fn decoder(_header: &[u8]) -> crate::Result<Self::Decoder> {
        Ok(())
    }

    fn decompress_page_into(
        _decoder: &mut Self::Decoder,
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> crate::Result<()> {
        let decompressed = decode_all(body)?;
        Self::bytes_to_values_into(&decompressed, expected_len, dst)
    }
}
