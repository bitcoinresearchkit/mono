use crate::{BytesVecValue, Version};

/// Packing policy for values stored inline when possible and in an overflow
/// sidecar otherwise.
///
/// Implementations must keep inline values and overflow pointers disjoint:
/// `to_compact` results must round-trip through `from_compact`, while a value
/// returned by `from_overflow_index(i)` must make `overflow_index` return
/// `Some(i)`.
pub trait OverflowVecValue: BytesVecValue {
    type Compact: BytesVecValue + Copy;

    /// Version of the compact representation and overflow-pointer encoding.
    const VERSION: Version;

    /// Returns the inline representation, or `None` when the full value must
    /// be stored in the overflow sidecar.
    fn to_compact(&self) -> Option<Self::Compact>;

    /// Decodes an inline value. This is only called when
    /// [`Self::overflow_index`] returned `None`.
    fn from_compact(compact: Self::Compact) -> Self;

    /// Returns the referenced sidecar index when `compact` is an overflow
    /// pointer.
    fn overflow_index(compact: Self::Compact) -> Option<usize>;

    /// Encodes a sidecar index as an overflow pointer.
    fn from_overflow_index(index: usize) -> Self::Compact;
}
