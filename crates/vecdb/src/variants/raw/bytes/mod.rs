use super::ReadWriteRawVec;
use crate::cache::NoCache;

pub mod reader;
pub mod strategy;
pub mod value;

pub use reader::*;
pub use strategy::*;
pub use value::*;

/// Raw storage vector using explicit byte serialization in little-endian format.
///
/// Uses the `Bytes` trait to serialize values with `to_bytes()/from_bytes()` in
/// **LITTLE-ENDIAN** format, ensuring **portability across different endianness systems**.
///
/// Like `ZeroCopyVec`, this is an append-only raw vector with push, truncate,
/// and rollback support. Wrap it in [`MutableVec`](crate::MutableVec) when
/// existing values must be updated or deleted.
///
/// The only difference from `ZeroCopyVec` is the serialization strategy:
/// - `BytesVec`: Explicit little-endian, portable across architectures
/// - `ZeroCopyVec`: Native byte order, faster but not portable
///
/// Use `BytesVec` when:
/// - Sharing data between systems with different endianness
/// - Cross-platform compatibility is required
/// - Custom serialization logic is needed
///
/// Use `ZeroCopyVec` when:
/// - Maximum performance is critical
/// - Data stays on the same architecture
pub type BytesVec<I, T, C = NoCache> = ReadWriteRawVec<I, T, BytesStrategy<T>, C>;
