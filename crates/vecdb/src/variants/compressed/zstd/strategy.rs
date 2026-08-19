use std::marker::PhantomData;

use zstd::{decode_all, encode_all};

use crate::impl_bytes_value_strategy;

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
    fn compress(values: &[T]) -> crate::Result<Vec<u8>> {
        let bytes = Self::values_to_bytes(values);
        Ok(encode_all(bytes.as_slice(), ZSTD_COMPRESSION_LEVEL)?)
    }

    fn decompress(bytes: &[u8], expected_len: usize) -> crate::Result<Vec<T>> {
        let decompressed = decode_all(bytes)?;
        Self::bytes_to_values(&decompressed, expected_len)
    }

    fn decompress_into(bytes: &[u8], expected_len: usize, dst: &mut Vec<T>) -> crate::Result<()> {
        let decompressed = decode_all(bytes)?;
        Self::bytes_to_values_into(&decompressed, expected_len, dst)
    }
}
