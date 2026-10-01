use crate::{
    Format, ReadOnlyRawVec, VecIndex, VecValue,
    cache::{CachePolicy, NoCache},
};

use super::ReadWriteRawVec;

pub mod strategy;
pub mod value;

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
pub struct ZeroCopyVec<I, T: VecValue, C: CachePolicy = NoCache>(
    ReadWriteRawVec<I, T, ZeroCopyStrategy<T>, C>,
);

impl<I, T, C: CachePolicy> ZeroCopyVec<I, T, C>
where
    I: VecIndex,
    T: ZeroCopyVecValue,
{
}

impl_vec_wrapper!(
    ZeroCopyVec,
    ReadWriteRawVec<I, T, ZeroCopyStrategy<T>, C>,
    ZeroCopyVecValue,
    Format::ZeroCopy,
    ReadOnlyRawVec<I, T, ZeroCopyStrategy<T>, C>,
    no_deref_mut
);

impl_mutable_raw_vec!(ZeroCopyVec, ZeroCopyVecValue, ZeroCopyStrategy, VecReader<I, T, ZeroCopyStrategy<T>>);
