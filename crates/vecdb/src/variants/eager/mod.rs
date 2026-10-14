use crate::{StoredVec, traits::writable::MAX_CACHE_SIZE};

pub mod any_stored_vec;
pub mod any_vec;
pub mod compute;
pub mod importable;
pub mod readable;
pub mod readable_cloneable;
pub mod stored;
pub mod typed;
pub mod writable;

/// Wrapper for computing and storing derived values from source vectors.
///
/// `EagerVec` wraps any `StoredVec` and provides computation methods to derive and persist
/// calculated values. Results are stored on disk and automatically recomputed when:
/// - Source data versions change
/// - The vector's computation logic version changes
///
/// # Key Features
/// - **Incremental Updates**: Only computes missing values, not the entire dataset
/// - **Automatic Versioning**: Detects stale data and recomputes automatically
/// - **Batched Writes**: Flushes periodically to prevent excessive memory usage
///
/// # Common Operations
/// - Transformations: `compute_transform()`, `compute_batched_to()`
/// - Arithmetic: `compute_subtract()`
/// - Custom computations: `compute_to()` (rolling statistics build on it downstream)
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
pub struct EagerVec<V>(V);

impl<V> EagerVec<V>
where
    V: StoredVec,
{
    /// Values per batch: enough that `pushed_len * SIZE_OF_T >= MAX_CACHE_SIZE`, so
    /// `batch_limit_reached()` fires.
    #[inline]
    fn batch_capacity(&self) -> usize {
        let size = size_of::<V::T>().max(1);
        MAX_CACHE_SIZE.div_ceil(size)
    }
}
