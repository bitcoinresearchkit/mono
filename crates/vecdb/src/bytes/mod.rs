use crate::Result;

pub mod array;
pub mod numeric;

/// Fixed-width serialization with an explicit byte order: numeric primitives are little-endian,
/// so stored data is portable across endianness (`ZeroCopyVec` stores native layout instead).
///
/// # Safety
///
/// `to_bytes` must return exactly `size_of::<Self>()` bytes, whatever `IS_NATIVE_LAYOUT`:
/// vectors store that many bytes per value and read rows back at those offsets unchecked.
///
/// When `IS_NATIVE_LAYOUT` is true, `Self` must also have no padding, `to_bytes` must
/// return exactly the value's in-memory bytes, and every bit pattern of that size must be
/// a valid `Self` (the value `from_bytes` would return). Vectors then copy values to and
/// from storage without `to_bytes` or `from_bytes`.
pub unsafe trait Bytes: Sized {
    type Array: AsRef<[u8]>;

    /// Whether the byte representation from `to_bytes` is identical to the
    /// in-memory representation of Self. When true, bulk operations can use
    /// memcpy instead of per-element deserialization (see the trait's safety contract).
    ///
    /// For numeric types, this is true on little-endian platforms (since
    /// `to_bytes`/`from_bytes` use little-endian format which matches native).
    const IS_NATIVE_LAYOUT: bool = false;

    fn to_bytes(&self) -> Self::Array;

    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}
