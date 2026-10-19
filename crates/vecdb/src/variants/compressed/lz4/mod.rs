use crate::{ReadWriteCompressedVec, cache::NoCache};

pub mod strategy;

pub use strategy::*;

/// Compressed vector: LZ4 over [`Bytes`](crate::Bytes) serialization.
pub type LZ4Vec<I, T, C = NoCache> = ReadWriteCompressedVec<I, T, LZ4Strategy<T>, C>;
