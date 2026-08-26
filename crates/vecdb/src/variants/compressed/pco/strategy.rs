use std::marker::PhantomData;

use pco::ChunkConfig;
use pco::standalone::{simple_compress, simple_decompress, simple_decompress_into};

use crate::{Error, impl_bytes_value_strategy, likely};

use super::{
    super::inner::CompressionStrategy,
    value::{AsInnerSlice, FromInnerSlice, PcoVecValue},
};

/// Pcodec compression strategy for numerical data.
#[derive(Debug, Clone, Copy)]
pub struct PcodecStrategy<T>(PhantomData<T>);

impl_bytes_value_strategy!(PcodecStrategy, PcoVecValue);

impl<T> CompressionStrategy<T> for PcodecStrategy<T>
where
    T: PcoVecValue,
{
    fn compress(values: &[T]) -> crate::Result<Vec<u8>> {
        let config = ChunkConfig::default()
            .with_compression_level(6)
            .with_enable_8_bit(true);
        Ok(simple_compress(values.as_inner_slice(), &config)?)
    }

    fn decompress(bytes: &[u8], expected_len: usize) -> crate::Result<Vec<T>> {
        let vec: Vec<T::NumberType> = simple_decompress(bytes)?;
        let vec = T::from_inner_slice(vec);

        if likely(vec.len() == expected_len) {
            return Ok(vec);
        }

        Err(Error::DecompressionMismatch {
            expected_len,
            actual_len: vec.len(),
        })
    }

    #[inline]
    fn decompress_into(bytes: &[u8], expected_len: usize, dst: &mut Vec<T>) -> crate::Result<()> {
        dst.clear();
        Self::decompress_append(bytes, expected_len, dst)
    }

    #[inline]
    fn decompress_append(bytes: &[u8], expected_len: usize, dst: &mut Vec<T>) -> crate::Result<()> {
        dst.reserve(expected_len);
        let old_len = dst.len();

        // SAFETY: MaybeUninit<T::NumberType> has the same layout as T::NumberType.
        // simple_decompress_into will initialize the memory, and we only set_len
        // after initialization succeeds.
        let spare = dst.spare_capacity_mut();
        let slice = unsafe {
            std::slice::from_raw_parts_mut(spare.as_mut_ptr().cast::<T::NumberType>(), expected_len)
        };

        let progress = simple_decompress_into(bytes, slice)?;

        // SAFETY: simple_decompress_into initialized progress.n_processed elements.
        unsafe {
            dst.set_len(old_len + progress.n_processed);
        }

        if likely(progress.n_processed == expected_len) {
            return Ok(());
        }

        Err(Error::DecompressionMismatch {
            expected_len,
            actual_len: progress.n_processed,
        })
    }
}
