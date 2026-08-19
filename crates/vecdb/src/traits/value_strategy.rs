/// Value serialization strategy shared by all vec types (raw and compressed).
///
/// Handles reading/writing individual values to/from bytes.
pub trait ValueStrategy<T>: Send + Sync + Clone {
    /// Whether T has native memory layout (can be memcpy'd to/from bytes).
    const IS_NATIVE_LAYOUT: bool = false;

    /// Deserializes a value from its byte representation.
    fn read(bytes: &[u8]) -> crate::Result<T>;

    /// Serializes a value by appending its byte representation to the buffer.
    fn write_to_vec(value: &T, buf: &mut Vec<u8>);

    /// Serializes a value directly into a fixed-size slice.
    fn write_to_slice(value: &T, dst: &mut [u8]);
}

/// Implements `ValueStrategy` for a Bytes-based strategy type.
/// Used by all Bytes-based strategies (BytesStrategy, LZ4Strategy, ZstdStrategy, PcodecStrategy).
macro_rules! impl_bytes_value_strategy {
    ($strategy:ident, $value_trait:path) => {
        impl<T> $crate::ValueStrategy<T> for $strategy<T>
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

pub(crate) use impl_bytes_value_strategy;
