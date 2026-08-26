use rawdb::likely;

use crate::{Error, ValueStrategy};

use super::EncodedChunk;

/// Trait for compression strategies used by ReadWriteCompressedVec.
pub trait CompressionStrategy<T>: ValueStrategy<T> {
    type Decoder;

    /// Maximum amount of uncompressed data that should share one compression context.
    const MAX_UNCOMPRESSED_CHUNK_SIZE: usize;

    /// Compresses full pages into one shared compression context.
    fn compress_chunk(values: &[T], values_per_page: usize) -> crate::Result<EncodedChunk>;

    /// Builds the reusable decoder for one chunk header.
    fn decoder(header: &[u8]) -> crate::Result<Self::Decoder>;

    /// Decodes one page body, replacing `dst` while reusing its allocation.
    fn decompress_page_into(
        decoder: &mut Self::Decoder,
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> crate::Result<()>;

    /// Decodes one page body directly onto the end of `dst`.
    #[inline]
    fn decompress_page_append(
        decoder: &mut Self::Decoder,
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> crate::Result<()> {
        let mut values = Vec::with_capacity(expected_len);
        Self::decompress_page_into(decoder, body, expected_len, &mut values)?;
        dst.extend(values);
        Ok(())
    }

    /// Serializes a slice of values to bytes.
    #[inline]
    fn values_to_bytes(values: &[T]) -> Vec<u8> {
        let byte_len = size_of_val(values);
        let mut bytes = Vec::with_capacity(byte_len);
        if Self::IS_NATIVE_LAYOUT {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    values.as_ptr() as *const u8,
                    bytes.as_mut_ptr(),
                    byte_len,
                );
                bytes.set_len(byte_len);
            }
        } else {
            for v in values {
                Self::write_to_vec(v, &mut bytes);
            }
        }
        bytes
    }

    /// Deserializes bytes into an existing buffer, reusing its allocation.
    #[inline]
    fn bytes_to_values_into(
        bytes: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> crate::Result<()> {
        let expected_bytes = expected_len * size_of::<T>();
        dst.clear();
        dst.reserve(expected_len);
        if !likely(bytes.len() == expected_bytes) {
            return Err(Error::DecompressionMismatch {
                expected_len,
                actual_len: bytes.len() / size_of::<T>(),
            });
        }
        if Self::IS_NATIVE_LAYOUT {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    dst.as_mut_ptr() as *mut u8,
                    expected_bytes,
                );
                dst.set_len(expected_len);
            }
        } else {
            let value_size = size_of::<T>();
            for index in 0..expected_len {
                let from = index * value_size;
                dst.push(Self::read(&bytes[from..from + value_size])?);
            }
        }
        Ok(())
    }
}
