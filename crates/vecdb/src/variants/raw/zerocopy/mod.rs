use crate::cache::NoCache;

use super::ReadWriteRawVec;

pub mod strategy;
pub mod value;

pub use strategy::*;
pub use value::*;

/// Raw vector storing values in native layout through zerocopy: not portable across endianness
/// (use [`BytesVec`](crate::BytesVec) for that).
pub type ZeroCopyVec<I, T, C = NoCache> = ReadWriteRawVec<I, T, ZeroCopyStrategy<T>, C>;
