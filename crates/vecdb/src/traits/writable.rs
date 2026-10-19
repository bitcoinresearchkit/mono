use std::{collections::BTreeMap, fs::File, io::Read, path::PathBuf};

use log::debug;

use crate::{AnyStoredVec, Bytes, Error, Result, Stamp, VecIndex, VecValue, Version};

/// Pushed bytes per write batch (1 GiB): `compute_to` batches by it and
/// [`WritableVec::batch_limit_reached`] reports reaching it; nothing writes on its own.
pub const MAX_CACHE_SIZE: usize = 1024 * 1024 * 1024;

/// Typed interface for stored vectors (push, truncate, rollback).
pub trait WritableVec<I, T>: AnyStoredVec
where
    I: VecIndex,
    T: VecValue,
{
    const SIZE_OF_T: usize = size_of::<T>();

    fn push(&mut self, value: T);

    /// Returns the current pushed (uncommitted) values.
    fn pushed(&self) -> &[T];

    fn truncate_if_needed_at(&mut self, index: usize) -> Result<()>;

    fn reset(&mut self) -> Result<()>;

    fn reset_unsaved(&mut self);

    /// Returns true if there are uncommitted changes.
    fn is_dirty(&self) -> bool;

    /// Writes an update checkpoint, saving changes before overwriting values.
    /// Compute loops should use `stamped_write_maybe_with_changes(stamp, false)`
    /// for periodic writes and save rollback history only at the final update.
    #[inline]
    fn stamped_write_with_changes(&mut self, stamp: Stamp) -> Result<()> {
        self.any_stamped_write_maybe_with_changes(stamp, true)
    }

    /// Rolls back the most recent change set.
    fn rollback(&mut self) -> Result<()>;

    #[doc(hidden)]
    fn find_rollback_files(&self) -> Result<BTreeMap<Stamp, PathBuf>>;

    /// Saves type-specific rollback state after the rollback loop completes.
    #[doc(hidden)]
    fn save_rollback_state(&mut self);

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

    /// Rolls back changes to before the given stamp and returns the stamp it lands at. The next
    /// write persists the restored state; until then read-only clones see the reduced length over
    /// the written bytes (see [`StoredVec::read_only_clone`](crate::StoredVec::read_only_clone)).
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

    fn truncate_if_needed(&mut self, index: I) -> Result<()> {
        self.truncate_if_needed_at(index.to_usize())
    }

    /// Truncates the vector to the given index if needed, updating the stamp.
    #[inline]
    fn truncate_if_needed_with_stamp(&mut self, index: I, stamp: Stamp) -> Result<()> {
        self.update_stamp(stamp);
        self.truncate_if_needed(index)
    }

    #[inline]
    fn clear(&mut self) -> Result<()> {
        self.truncate_if_needed_at(0)
    }

    /// Whether the pushed values fill a 1 GiB write batch: write before pushing more.
    #[inline]
    fn batch_limit_reached(&self) -> bool {
        self.pushed_len() * Self::SIZE_OF_T >= MAX_CACHE_SIZE
    }

    /// Writes a checkpoint at `stamp`, saving its undo record when `with_changes`.
    #[inline]
    fn stamped_write_maybe_with_changes(&mut self, stamp: Stamp, with_changes: bool) -> Result<()> {
        self.any_stamped_write_maybe_with_changes(stamp, with_changes)
    }

    /// Validates the computed version against the stored version, resetting if they don't match.
    /// Automatically includes the vec's own version - only pass dependency versions.
    fn validate_computed_version_or_reset(&mut self, dep_version: Version) -> Result<()> {
        self.any_validate_computed_version_or_reset(dep_version)?;

        if self.is_empty() {
            debug!("Computing {}_to_{}...", self.index_type_name(), self.name())
        }

        Ok(())
    }
}
