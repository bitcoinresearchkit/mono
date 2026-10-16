use crate::{ReadWriteCompressedVec, cache::NoCache};

pub mod strategy;
pub mod value;

pub use strategy::*;
pub use value::*;

/// Compressed storage using Zstd for maximum general-purpose compression.
///
/// Zstd (Zstandard) provides the best compression ratios among general-purpose
/// algorithms, with good decompression speed. Ideal when storage is expensive.
///
/// # Performance Characteristics
/// - Highest compression ratios (typically 3-5x, better than LZ4)
/// - Fast decompression (slower compression than LZ4)
/// - Works well with any data type
///
/// # When to Use
/// - Storage space is expensive
/// - Can tolerate slower compression (decompression is fast)
pub type ZstdVec<I, T, C = NoCache> = ReadWriteCompressedVec<I, T, ZstdStrategy<T>, C>;
