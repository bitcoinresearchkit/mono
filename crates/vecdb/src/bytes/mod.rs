mod array;
mod numeric;

/// Trait for types that can be serialized to/from bytes with explicit byte order.
///
/// This trait uses **LITTLE-ENDIAN** byte order for all numeric types, making the data
/// **portable across different endianness systems** (x86, ARM, etc.). This is the key
/// difference from `ZeroCopyVec`, which uses native byte order and is not portable.
///
/// Use this trait when:
/// - You need cross-platform compatibility
/// - You're sharing data between systems with different endianness
/// - You need custom serialization logic
///
/// For maximum performance on a single system, use `ZeroCopyVec` instead.
pub trait Bytes: Sized {
    /// The byte array type returned by `to_bytes`.
    /// For fixed-size types, this is `[u8; N]` where N is the size of the type.
    type Array: AsRef<[u8]>;

    /// Whether the byte representation from `to_bytes` is identical to the
    /// in-memory representation of Self. When true, bulk operations can use
    /// memcpy instead of per-element deserialization.
    ///
    /// For numeric types, this is true on little-endian platforms (since
    /// `to_bytes`/`from_bytes` use little-endian format which matches native).
    const IS_NATIVE_LAYOUT: bool = false;

    /// Serializes this value to bytes.
    ///
    /// For numeric types, this uses little-endian byte order (via `to_le_bytes`).
    fn to_bytes(&self) -> Self::Array;

    /// Deserializes a value from bytes.
    ///
    /// For numeric types, this uses little-endian byte order (via `from_le_bytes`).
    fn from_bytes(bytes: &[u8]) -> crate::Result<Self>;
}
