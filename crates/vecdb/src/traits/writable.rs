use std::{collections::BTreeMap, fs::File, io::Read, path::PathBuf};

use log::debug;

use crate::{AnyStoredVec, Bytes, Error, Result, Stamp, VecIndex, VecValue, Version};

/// Maximum in-memory cache size before forcing a flush (1 GiB).
/// Prevents unbounded memory growth when pushing many values without flushing.
pub const MAX_CACHE_SIZE: usize = 1024 * 1024 * 1024;

/// Typed interface for stored vectors (push, truncate, rollback).
///
/// Provides the core write operations for all stored vec types.
/// For reading, use [`ReadableVec`] (`collect_range`, `fold_range_at`, etc.).
/// Raw vecs (`BytesVec`, `ZeroCopyVec`) additionally provide
/// `VecReader` for O(1) random access.
///
/// [`ReadableVec`]: crate::ReadableVec
pub trait WritableVec<I, T>: AnyStoredVec
where
    I: VecIndex,
    T: VecValue,
{
    const SIZE_OF_T: usize = size_of::<T>();

    fn push(&mut self, value: T);

    /// Returns the current pushed (uncommitted) values.
    fn pushed(&self) -> &[T];

    /// Truncates the vector to the given usize index if the current length exceeds it.
    fn truncate_if_needed_at(&mut self, index: usize) -> Result<()>;

    /// Resets the vector state.
    fn reset(&mut self) -> Result<()>;

    /// Resets uncommitted changes.
    fn reset_unsaved(&mut self);

    /// Returns true if there are uncommitted changes.
    fn is_dirty(&self) -> bool;

    /// Writes an update checkpoint, saving changes before overwriting values.
    /// Compute loops should use `stamped_write_maybe_with_changes(stamp, false)`
    /// for periodic writes and save rollback history only at the final update.
    fn stamped_write_with_changes(&mut self, stamp: Stamp) -> Result<()>;

    /// Rolls back the most recent change set.
    fn rollback(&mut self) -> Result<()>;

    /// Returns available rollback change files.
    #[doc(hidden)]
    fn find_rollback_files(&self) -> Result<BTreeMap<Stamp, PathBuf>>;

    /// Saves type-specific rollback state after the rollback loop completes.
    #[doc(hidden)]
    fn save_rollback_state(&mut self);

    /// Rolls back changes to before the given stamp.
    /// The stamp [`Self::rollback_before`] lands at for `stamp`, without rolling back; `None`
    /// when the saved changes do not reach below `stamp`.
    fn rollback_landing(&self, stamp: Stamp) -> Result<Option<Stamp>> {
        let mut current = self.stamp();
        if current < stamp {
            return Ok(Some(current));
        }
        let files = self.find_rollback_files()?;
        while current >= stamp {
            let Some(path) = files.get(&current) else {
                return Ok(None);
            };
            // A change file opens with the stamp its rollback restores.
            let mut previous = [0; size_of::<u64>()];
            File::open(path)?.read_exact(&mut previous)?;
            let previous = Stamp::from_bytes(&previous)?;
            // A link that does not go down (a record saved at its own stamp) ends the chain.
            if previous >= current {
                return Ok(None);
            }
            current = previous;
        }
        Ok(Some(current))
    }

    fn rollback_before(&mut self, stamp: Stamp) -> Result<Stamp> {
        self.header().check_writable()?;
        let files = self.find_rollback_files()?;

        // Walk change files newest-first. Each rollback decrements the vec
        // stamp to the previous file's stamp. Files above it are left from an
        // earlier rollback (kept until the next write replaces or removes them);
        // a missing one below it means the chain is broken.
        let mut undone = false;
        for (&file_stamp, _) in files.range(..=self.stamp()).rev() {
            let current = self.stamp();
            if current < stamp {
                break;
            }
            if file_stamp > current {
                continue;
            }
            if file_stamp != current {
                return Err(Error::StampMismatch {
                    file: file_stamp,
                    vec: current,
                });
            }
            self.rollback()?;
            undone = true;
        }

        // A no-op rollback keeps the existing baseline: saving here would snapshot unsaved mutations.
        if undone {
            self.save_rollback_state();
        }
        Ok(self.stamp())
    }

    /// Number of pushed (uncommitted) values in the memory buffer.
    #[inline]
    fn pushed_len(&self) -> usize {
        self.pushed().len()
    }

    /// Pushes a value and checks its expected index in debug builds.
    #[inline]
    fn debug_checked_push(&mut self, index: I, value: T) {
        debug_assert_eq!(
            index.to_usize(),
            self.len(),
            "unexpected index for {}",
            self.name(),
        );
        self.push(value);
    }

    /// Truncates the vector to the given index if the current length exceeds it.
    fn truncate_if_needed(&mut self, index: I) -> Result<()> {
        self.truncate_if_needed_at(index.to_usize())
    }

    /// Truncates the vector to the given index if needed, updating the stamp.
    #[inline]
    fn truncate_if_needed_with_stamp(&mut self, index: I, stamp: Stamp) -> Result<()> {
        self.update_stamp(stamp);
        self.truncate_if_needed(index)
    }

    /// Clears all values from the vector.
    #[inline]
    fn clear(&mut self) -> Result<()> {
        self.truncate_if_needed_at(0)
    }

    /// Returns true if the pushed cache has reached the batch limit (~1GiB).
    ///
    /// When this limit is reached, the caller should flush to disk before continuing.
    /// This prevents excessive memory usage during bulk operations.
    #[inline]
    fn batch_limit_reached(&self) -> bool {
        self.pushed_len() * Self::SIZE_OF_T >= MAX_CACHE_SIZE
    }

    /// Extends the vector to `target_len`, filling with `value`.
    /// Batches writes in ~1GB chunks to avoid memory explosion.
    fn fill_to(&mut self, target_len: usize, value: T) -> Result<()>
    where
        T: Copy,
    {
        let batch_count = MAX_CACHE_SIZE / Self::SIZE_OF_T.max(1);

        while self.len() < target_len {
            let count = (target_len - self.len()).min(batch_count);
            for _ in 0..count {
                self.push(value);
            }
            if self.batch_limit_reached() {
                self.write()?;
            }
        }
        Ok(())
    }

    /// Flushes with the given stamp, optionally saving changes for rollback.
    #[inline]
    fn stamped_write_maybe_with_changes(&mut self, stamp: Stamp, with_changes: bool) -> Result<()> {
        self.any_stamped_write_maybe_with_changes(stamp, with_changes)
    }

    /// Validates the computed version against the stored version, resetting if they don't match.
    /// Automatically includes the vec's own version - only pass dependency versions.
    fn validate_computed_version_or_reset(&mut self, dep_version: Version) -> Result<()> {
        self.any_validate_computed_version_or_reset(dep_version)?;

        if self.is_empty() {
            debug!(
                "Computing {}_to_{}...",
                self.index_type_to_string(),
                self.name()
            )
        }

        Ok(())
    }
}
