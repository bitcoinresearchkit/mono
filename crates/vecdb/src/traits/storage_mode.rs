use crate::{ReadableVec, StoredVec, TypedVec};

/// Marker trait that selects between read-write and read-only storage.
///
/// Composite types use `M::Stored<V>` for stored vec fields.
/// When `M = Rw`, the field is `V` itself (identity) with full write access.
/// When `M = Ro`, the field is `V::ReadOnly`, its read-only clone.
pub trait StorageMode: 'static {
    type Stored<V: StoredVec + 'static>: TypedVec<I = V::I, T = V::T>
        + ReadableVec<V::I, V::T>
        + 'static;

    /// Computation state present only in the writer: `T` in `Rw`, `()` in `Ro`.
    /// No wrapper, cloning, or default construction of `T` is required.
    type WriteOnly<T>;
}
