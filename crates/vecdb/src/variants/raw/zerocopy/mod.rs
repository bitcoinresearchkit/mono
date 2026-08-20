use crate::{AnyStoredVec, Format, ReadOnlyRawVec, VecIndex, VecReader, impl_vec_wrapper};

use super::ReadWriteRawVec;

mod strategy;
mod value;

pub use strategy::*;
pub use value::*;

/// Raw storage vector using zerocopy for direct memory mapping in native byte order.
///
/// Uses the `zerocopy` crate for direct memory-mapped access without copying, providing
/// the fastest possible performance. Values are stored in **NATIVE byte order**.
///
/// Like `BytesVec`, this is an append-only raw vector with push, truncate, and
/// rollback support. Wrap it in [`MutableVec`](crate::MutableVec) when
/// existing values must be updated or deleted.
///
/// The only difference from `BytesVec` is the serialization strategy:
/// - `ZeroCopyVec`: Native byte order, faster but not portable
/// - `BytesVec`: Explicit little-endian, portable across architectures
///
/// # Portability Warning
///
/// **NOT portable across systems with different endianness.** Data written on a
/// little-endian system (x86) cannot be read correctly on a big-endian system.
/// For portable storage, use `BytesVec` instead.
///
/// Use `ZeroCopyVec` when:
/// - Maximum performance is critical
/// - Data stays on the same architecture
///
/// Use `BytesVec` when:
/// - Cross-platform compatibility is needed
/// - Sharing data between different architectures
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
pub struct ZeroCopyVec<I, T>(pub(crate) ReadWriteRawVec<I, T, ZeroCopyStrategy<T>>);

impl<I, T> ZeroCopyVec<I, T>
where
    I: VecIndex,
    T: ZeroCopyVecValue,
{
    /// The size of T in bytes.
    pub const SIZE_OF_T: usize = size_of::<T>();

    /// Returns a reference to the value directly from the memory-mapped file without copying.
    /// Very efficient for large types or frequent reads.
    ///
    /// Returns `None` if:
    /// - Index is beyond stored length (might be in pushed layer)
    #[inline]
    pub fn read_ref<'a>(
        &self,
        index: I,
        reader: &'a VecReader<I, T, ZeroCopyStrategy<T>>,
    ) -> Option<&'a T> {
        self.read_ref_at(index.to_usize(), reader)
    }

    /// Returns a reference to the value at the given usize index directly from the memory-mapped file.
    #[inline]
    pub fn read_ref_at<'a>(
        &self,
        index: usize,
        reader: &'a VecReader<I, T, ZeroCopyStrategy<T>>,
    ) -> Option<&'a T> {
        let stored_len = reader.len();
        debug_assert_eq!(stored_len, self.stored_len(), "stale VecReader");

        // Cannot return ref for pushed values (they're in a Vec, not mmap)
        if index >= stored_len {
            return None;
        }

        self.stored_ref_at(index, reader)
    }

    #[inline]
    fn stored_ref_at<'a>(
        &self,
        index: usize,
        reader: &'a VecReader<I, T, ZeroCopyStrategy<T>>,
    ) -> Option<&'a T> {
        let offset = index * Self::SIZE_OF_T;
        let bytes = reader.as_bytes().get(offset..)?;
        T::ref_from_prefix(bytes).map(|(v, _)| v).ok()
    }
}

impl_vec_wrapper!(
    ZeroCopyVec,
    ReadWriteRawVec<I, T, ZeroCopyStrategy<T>>,
    ZeroCopyVecValue,
    Format::ZeroCopy,
    ReadOnlyRawVec<I, T, ZeroCopyStrategy<T>>,
    no_deref_mut
);
