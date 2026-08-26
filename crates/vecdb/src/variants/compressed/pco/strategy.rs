use std::{
    marker::PhantomData,
    mem::{MaybeUninit, align_of, size_of},
};

use pco::wrapped::{ChunkDecompressor, FileCompressor, FileDecompressor};
use pco::{ChunkConfig, PagingSpec};

use crate::{EncodedChunk, Error, impl_bytes_value_strategy, likely};

use super::{super::inner::CompressionStrategy, value::PcoVecValue};

/// Pcodec compression strategy for numerical data.
#[derive(Debug, Clone, Copy)]
pub struct PcodecStrategy<T>(PhantomData<T>);

impl_bytes_value_strategy!(PcodecStrategy, PcoVecValue);

impl<T> PcodecStrategy<T>
where
    T: PcoVecValue,
{
    #[inline]
    fn decode_uninit(
        decoder: &mut ChunkDecompressor<T::NumberType>,
        body: &[u8],
        expected_len: usize,
        dst: &mut [MaybeUninit<T::NumberType>],
    ) -> crate::Result<()> {
        debug_assert_eq!(dst.len(), expected_len);

        let mut page = decoder.page_decompressor(body, expected_len)?;
        let progress = page.read_uninit(dst)?;

        if likely(progress.n_processed == expected_len) {
            Ok(())
        } else {
            Err(Error::DecompressionMismatch {
                expected_len,
                actual_len: progress.n_processed,
            })
        }
    }

    #[inline]
    fn decode_numbers(
        decoder: &mut ChunkDecompressor<T::NumberType>,
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T::NumberType>,
    ) -> crate::Result<()> {
        dst.clear();
        dst.reserve(expected_len);
        Self::decode_uninit(
            decoder,
            body,
            expected_len,
            &mut dst.spare_capacity_mut()[..expected_len],
        )?;
        // SAFETY: decode_uninit initialized the complete spare-capacity slice.
        unsafe { dst.set_len(expected_len) };
        Ok(())
    }

    #[inline]
    fn decode_transparent_append(
        decoder: &mut ChunkDecompressor<T::NumberType>,
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> crate::Result<()> {
        debug_assert!(T::IS_TRANSPARENT);
        debug_assert_eq!(size_of::<T>(), size_of::<T::NumberType>());
        debug_assert_eq!(align_of::<T>(), align_of::<T::NumberType>());

        let initial_len = dst.len();
        dst.reserve(expected_len);
        let spare = &mut dst.spare_capacity_mut()[..expected_len];
        // SAFETY: The Pco contract guarantees identical layouts and valid bit
        // patterns when IS_TRANSPARENT is true. MaybeUninit preserves the
        // underlying value layout.
        let numbers = unsafe {
            std::slice::from_raw_parts_mut(
                spare.as_mut_ptr().cast::<MaybeUninit<T::NumberType>>(),
                spare.len(),
            )
        };
        Self::decode_uninit(decoder, body, expected_len, numbers)?;
        // SAFETY: decode_uninit initialized the complete spare-capacity slice
        // with NumberType values, which are valid T values by the Pco contract.
        unsafe { dst.set_len(initial_len + expected_len) };
        Ok(())
    }
}

impl<T> CompressionStrategy<T> for PcodecStrategy<T>
where
    T: PcoVecValue,
{
    type Decoder = (ChunkDecompressor<T::NumberType>, Vec<T::NumberType>);

    const MAX_UNCOMPRESSED_CHUNK_SIZE: usize = 128 * 1024;

    fn compress_chunk(values: &[T], values_per_page: usize) -> crate::Result<EncodedChunk> {
        let config = ChunkConfig::default()
            .with_compression_level(6)
            .with_enable_8_bit(true)
            .with_paging_spec(PagingSpec::EqualPagesUpTo(values_per_page));
        let file = FileCompressor::default();
        let mut bytes = file.write_header(Vec::new())?;
        let converted;
        let numbers = if T::IS_TRANSPARENT {
            debug_assert_eq!(size_of::<T>(), size_of::<T::NumberType>());
            debug_assert_eq!(align_of::<T>(), align_of::<T::NumberType>());
            // SAFETY: The Pco contract guarantees identical layouts and valid
            // bit patterns when IS_TRANSPARENT is true.
            unsafe {
                std::slice::from_raw_parts(values.as_ptr().cast::<T::NumberType>(), values.len())
            }
        } else {
            converted = values.iter().copied().map(T::to_number).collect::<Vec<_>>();
            &converted
        };
        let mut chunk = file.chunk_compressor(numbers, &config)?;
        bytes = chunk.write_meta(bytes)?;
        let header_len = bytes.len();
        let mut page_ends = Vec::with_capacity(chunk.n_per_page().len());
        for page_index in 0..chunk.n_per_page().len() {
            bytes = chunk.write_page(page_index, bytes)?;
            page_ends.push(u32::try_from(bytes.len()).map_err(|_| Error::Overflow)?);
        }
        EncodedChunk::new(bytes, header_len, page_ends)
    }

    fn decoder(header: &[u8]) -> crate::Result<Self::Decoder> {
        let (file, rest) = FileDecompressor::new(header)?;
        let (decoder, rest) = file.chunk_decompressor(rest)?;
        if !rest.is_empty() {
            return Err(Error::InvalidArgument("trailing PCO chunk header bytes"));
        }
        Ok((decoder, Vec::new()))
    }

    #[inline]
    fn decompress_page_into(
        decoder: &mut Self::Decoder,
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> crate::Result<()> {
        let (decompressor, numbers) = decoder;
        if T::IS_TRANSPARENT {
            dst.clear();
            return Self::decode_transparent_append(decompressor, body, expected_len, dst);
        }

        Self::decode_numbers(decompressor, body, expected_len, numbers)?;
        dst.clear();
        dst.reserve(expected_len);
        for &number in numbers.iter() {
            dst.push(T::from_number(number)?);
        }
        Ok(())
    }

    #[inline]
    fn decompress_page_append(
        decoder: &mut Self::Decoder,
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> crate::Result<()> {
        let (decompressor, numbers) = decoder;
        if T::IS_TRANSPARENT {
            return Self::decode_transparent_append(decompressor, body, expected_len, dst);
        }

        Self::decode_numbers(decompressor, body, expected_len, numbers)?;
        dst.reserve(expected_len);
        for &number in numbers.iter() {
            dst.push(T::from_number(number)?);
        }
        Ok(())
    }
}
