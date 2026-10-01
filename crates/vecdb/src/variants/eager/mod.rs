use brk_exit::Exit;
use log::debug;

use crate::{
    AnyStoredVec, AnyVec, Result, StoredVec, Version, WritableVec, traits::writable::MAX_CACHE_SIZE,
};

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
/// - Arithmetic: `compute_subtract()`, `compute_multiply()`
/// - Moving statistics: `compute_sma()`, `compute_rolling_ema()`, `compute_rolling_sum()`
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
pub struct EagerVec<V>(V);

impl<V> EagerVec<V>
where
    V: StoredVec,
{
    /// Validates version, truncates to `max_from`, then runs `f` in batched writes.
    fn compute_init<F>(&mut self, version: Version, max_from: V::I, exit: &Exit, f: F) -> Result<()>
    where
        F: FnMut(&mut Self) -> Result<()>,
    {
        {
            let _lock = exit.lock();
            self.validate_computed_version_or_reset(version)?;
            self.truncate_if_needed(max_from)?;
        }
        self.repeat_until_complete(exit, f)
    }

    /// Max end index for one batch, capped at `max_end`.
    /// Ensures `pushed_len * SIZE_OF_T >= MAX_CACHE_SIZE` so `batch_limit_reached()` fires.
    #[inline]
    pub fn batch_end(&self, max_end: usize) -> usize {
        self.len()
            .saturating_add(self.batch_capacity())
            .min(max_end)
    }

    #[inline]
    fn batch_capacity(&self) -> usize {
        let size = size_of::<V::T>().max(1);
        MAX_CACHE_SIZE.div_ceil(size)
    }

    /// Helper that repeatedly calls a compute function until it completes.
    /// Persists every successful batch, including a truncation-only final batch.
    pub fn repeat_until_complete<F>(&mut self, exit: &Exit, mut f: F) -> Result<()>
    where
        F: FnMut(&mut Self) -> Result<()>,
    {
        loop {
            f(self)?;
            let batch_limit_reached = self.batch_limit_reached();
            if batch_limit_reached {
                debug!("Batch limit reached, saving to disk...");
            }
            {
                let _lock = exit.lock();
                self.write()?;
            }
            if !batch_limit_reached {
                break;
            }
        }

        Ok(())
    }
}
