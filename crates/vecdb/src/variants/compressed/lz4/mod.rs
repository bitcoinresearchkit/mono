use crate::{ReadWriteCompressedVec, cache::NoCache};

pub mod strategy;

pub use strategy::*;

/// Compressed storage using LZ4 for speed-optimized general-purpose compression.
///
/// LZ4 prioritizes compression/decompression speed over ratio, making it ideal
/// for workloads where CPU time matters more than storage space.
///
/// # Performance Characteristics
/// - Extremely fast compression/decompression (hundreds of MB/s)
/// - Moderate compression ratios (typically 2-3x)
/// - Works well with any data type
///
/// # When to Use
/// - Speed is more important than storage savings
/// - Mixed data types (not just numbers)
/// - Need compression but can't afford CPU overhead
pub type LZ4Vec<I, T, C = NoCache> = ReadWriteCompressedVec<I, T, LZ4Strategy<T>, C>;
