use std::{borrow::Cow, slice};

use crate::Result;

/// Value serialization strategy shared by all vec types (raw and compressed).
///
/// Handles reading/writing individual values to/from bytes.
///
/// # Safety
///
/// Every value serializes to exactly `size_of::<T>()` bytes, whatever `IS_NATIVE_LAYOUT`:
/// `write_to_vec` appends that many, `write_to_slice` fills that many and `values_bytes`
/// returns that many per value. Vectors read rows back at those offsets unchecked.
///
/// When `IS_NATIVE_LAYOUT` is true, `T` must also meet the native-layout contract of
/// [`Bytes`](crate::Bytes): vectors then copy values to and from storage bytes directly.
pub unsafe trait ValueStrategy<T>: Send + Sync + Clone {
    /// Whether T has native memory layout (can be memcpy'd to/from bytes).
    const IS_NATIVE_LAYOUT: bool = false;

    /// Deserializes a value from its byte representation.
    fn read(bytes: &[u8]) -> Result<T>;

    /// Serializes a value by appending its byte representation to the buffer.
    fn write_to_vec(value: &T, buf: &mut Vec<u8>);

    /// Serializes a value directly into a fixed-size slice.
    fn write_to_slice(value: &T, dst: &mut [u8]);

    /// The stored bytes of `values`: borrowed for native layouts, else serialized.
    #[inline]
    fn values_bytes(values: &[T]) -> Cow<'_, [u8]> {
        if Self::IS_NATIVE_LAYOUT {
            // SAFETY: native-layout values are their own byte image (see `IS_NATIVE_LAYOUT`).
            Cow::Borrowed(unsafe {
                slice::from_raw_parts(values.as_ptr().cast(), size_of_val(values))
            })
        } else {
            let mut bytes = Vec::with_capacity(size_of_val(values));
            for value in values {
                Self::write_to_vec(value, &mut bytes);
            }
            debug_assert_eq!(
                bytes.len(),
                size_of_val(values),
                "values must serialize to their size (see `ValueStrategy`'s safety contract)"
            );
            Cow::Owned(bytes)
        }
    }
}

/// Implements `ValueStrategy` for a Bytes-based strategy type.
/// Used by all Bytes-based strategies (BytesStrategy, LZ4Strategy, ZstdStrategy, PcodecStrategy).
macro_rules! impl_bytes_value_strategy {
    ($strategy:ident, $value_trait:path) => {
        // SAFETY: values serialize through `T::to_bytes` (`size_of::<T>()` bytes) and the
        // native-layout flag is `T`'s, under the `Bytes` contract.
        unsafe impl<T> $crate::ValueStrategy<T> for $strategy<T>
        where
            T: $value_trait,
        {
            const IS_NATIVE_LAYOUT: bool = T::IS_NATIVE_LAYOUT;

            #[inline(always)]
            fn read(bytes: &[u8]) -> $crate::Result<T> {
                T::from_bytes(bytes)
            }

            #[inline(always)]
            fn write_to_vec(value: &T, buf: &mut Vec<u8>) {
                buf.extend_from_slice(value.to_bytes().as_ref());
            }

            #[inline(always)]
            fn write_to_slice(value: &T, dst: &mut [u8]) {
                dst.copy_from_slice(value.to_bytes().as_ref());
            }
        }
    };
}
