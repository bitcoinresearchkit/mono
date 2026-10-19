use crate::{ReadWriteCompressedVec, cache::NoCache};

pub mod strategy;

pub use strategy::*;

/// Compressed vector: Zstd over [`Bytes`](crate::Bytes) serialization.
pub type ZstdVec<I, T, C = NoCache> = ReadWriteCompressedVec<I, T, ZstdStrategy<T>, C>;
