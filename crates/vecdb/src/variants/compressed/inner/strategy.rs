use std::ptr;

use super::EncodedChunk;
use crate::{Error, Format, Result, ValueStrategy, likely};

pub trait CompressionStrategy<T>: ValueStrategy<T> + 'static {
    /// On-disk format of vectors stored with this strategy.
    const FORMAT: Format;

    /// Decoding state reused across pages, its scratch across chunks too.
    type Decoder: Default;

    /// Maximum amount of uncompressed data that should share one compression context.
    const MAX_UNCOMPRESSED_CHUNK_SIZE: usize;

    /// Compresses full pages into one shared compression context.
    fn compress_chunk(values: &[T], values_per_page: usize) -> Result<EncodedChunk>;

    /// Points `decoder` at the chunk with this header, keeping its reusable scratch. Pages are
    /// decoded only after a successful reset for their chunk.
    fn reset_decoder(decoder: &mut Self::Decoder, header: &[u8]) -> Result<()>;

    fn decompress_page_append(
        decoder: &mut Self::Decoder,
        body: &[u8],
        expected_len: usize,
        dst: &mut Vec<T>,
    ) -> Result<()>;

    #[inline]
    fn bytes_to_values_append(bytes: &[u8], expected_len: usize, dst: &mut Vec<T>) -> Result<()> {
        let expected_bytes = expected_len * size_of::<T>();
        if !likely(bytes.len() == expected_bytes) {
            return Err(Error::DecompressionMismatch {
                expected_len,
                actual_len: bytes.len() / size_of::<T>(),
            });
        }
        dst.reserve(expected_len);
        if Self::IS_NATIVE_LAYOUT {
            let len = dst.len();
            // SAFETY: native-layout values are their own byte image (`ValueStrategy` contract),
            // and the reserved spare capacity holds `expected_len` values.
            unsafe {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    dst.as_mut_ptr().add(len).cast::<u8>(),
                    expected_bytes,
                );
                dst.set_len(len + expected_len);
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
