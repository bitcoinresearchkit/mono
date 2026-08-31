use log::debug;

mod any_stored_vec;
mod any_vec;
mod compute;
mod importable;
mod readable;
mod readable_cloneable;
mod stored;
mod typed;
mod writable;

use crate::{
    AnyStoredVec, AnyVec, StoredVec, Version, WritableVec, traits::writable::MAX_CACHE_SIZE,
};
use brk_exit::Exit;

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
/// - Transformations: `compute_transform()`, `compute_range()`
/// - Arithmetic: `compute_add()`, `compute_subtract()`, `compute_multiply()`, `compute_divide()`
/// - Moving statistics: `compute_sma()`, `compute_ema()`, `compute_sum()`, `compute_max()`, `compute_min()`
/// - Lookback calculations: `compute_change()`, `compute_percentage_change()`
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
pub struct EagerVec<V>(pub(super) V);

impl<V> EagerVec<V>
where
    V: StoredVec,
{
    /// Validates version, truncates to `max_from`, then runs `f` in batched writes.
    fn compute_init<F>(
        &mut self,
        version: Version,
        max_from: V::I,
        exit: &Exit,
        f: F,
    ) -> crate::Result<()>
    where
        F: FnMut(&mut Self) -> crate::Result<()>,
    {
        self.validate_computed_version_or_reset(version)?;
        self.truncate_if_needed(max_from)?;
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
    /// Writes between iterations when batch limit is hit.
    pub fn repeat_until_complete<F>(&mut self, exit: &Exit, mut f: F) -> crate::Result<()>
    where
        F: FnMut(&mut Self) -> crate::Result<()>,
    {
        loop {
            f(self)?;
            let batch_limit_reached = self.batch_limit_reached();
            if batch_limit_reached {
                debug!("Batch limit reached, saving to disk...");
            }
            if self.is_dirty() {
                let _lock = exit.lock();
                self.write()?;
            }
            if !batch_limit_reached {
                break;
            }
        }

        Ok(())
    }

    /// Removes this vector and all its associated regions from the database
    pub fn remove(self) -> crate::Result<()> {
        self.0.remove()
    }
}
