use super::ReadWriteRawVec;
use crate::cache::NoCache;

pub mod reader;
pub mod strategy;
pub mod value;

pub use reader::*;
pub use strategy::*;
pub use value::*;

/// Raw vector serializing values through [`Bytes`](crate::Bytes): portable across endianness.
pub type BytesVec<I, T, C = NoCache> = ReadWriteRawVec<I, T, BytesStrategy<T>, C>;
