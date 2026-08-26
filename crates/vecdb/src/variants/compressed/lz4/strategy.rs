use std::marker::PhantomData;

use lz4_flex::{compress_prepend_size, decompress_size_prepended};

use crate::{EncodedChunk, impl_bytes_value_strategy};

use super::{super::inner::CompressionStrategy, value::LZ4VecValue};

/// LZ4 compression strategy for fast compression/decompression.
#[derive(Debug, Clone, Copy)]
pub struct LZ4Strategy<T>(PhantomData<T>);

impl_bytes_value_strategy!(LZ4Strategy, LZ4VecValue);

impl<T> CompressionStrategy<T> for LZ4Strategy<T>
where
    T: LZ4VecValue,
{
    type Decoder = ();

    const MAX_UNCOMPRESSED_CHUNK_SIZE: usize = 8 * 1024;

    fn compress_chunk(values: &[T], _values_per_page: usize) -> crate::Result<EncodedChunk> {
        EncodedChunk::single_page(compress_prepend_size(&Self::values_to_bytes(values)))
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
        let decompressed = decompress_size_prepended(body)?;
        Self::bytes_to_values_into(&decompressed, expected_len, dst)
    }
}
