use crate::{ReadWriteCompressedVec, cache::NoCache};

pub mod strategy;
pub mod r#trait;
pub mod value;

pub use strategy::*;
pub use r#trait::*;
pub use value::*;

/// Compressed storage using Pcodec for optimal numeric data compression.
///
/// Pcodec (Pco) provides the best compression ratios for numerical sequences
/// through specialized quantization and encoding. Ideal for time-series, scientific
/// data, and any workload dominated by numeric values.
///
/// # Performance Characteristics
/// - Best-in-class compression for numeric types (often 2-10x better than LZ4/Zstd)
/// - Sequential access optimized (values compressed in pages)
/// - Random access possible but slower than raw formats
///
/// # When to Use
/// - Numeric data dominates (integers, floats)
/// - Storage space is critical
/// - Sequential access patterns are common
pub type PcoVec<I, T, C = NoCache> = ReadWriteCompressedVec<I, T, PcodecStrategy<T>, C>;
