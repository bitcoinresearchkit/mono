use std::marker::PhantomData;

use lz4_flex::{compress_prepend_size, decompress_size_prepended};

use crate::impl_bytes_value_strategy;

use super::{super::inner::CompressionStrategy, value::LZ4VecValue};

/// LZ4 compression strategy for fast compression/decompression.
#[derive(Debug, Clone, Copy)]
pub struct LZ4Strategy<T>(PhantomData<T>);

impl_bytes_value_strategy!(LZ4Strategy, LZ4VecValue);

impl<T> CompressionStrategy<T> for LZ4Strategy<T>
where
    T: LZ4VecValue,
{
    fn compress(values: &[T]) -> crate::Result<Vec<u8>> {
        Ok(compress_prepend_size(&Self::values_to_bytes(values)))
    }

    fn decompress(bytes: &[u8], expected_len: usize) -> crate::Result<Vec<T>> {
        let decompressed = decompress_size_prepended(bytes)?;
        Self::bytes_to_values(&decompressed, expected_len)
    }

    fn decompress_into(bytes: &[u8], expected_len: usize, dst: &mut Vec<T>) -> crate::Result<()> {
        let decompressed = decompress_size_prepended(bytes)?;
        Self::bytes_to_values_into(&decompressed, expected_len, dst)
    }
}
