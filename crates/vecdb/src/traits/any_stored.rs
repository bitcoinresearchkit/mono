use std::path::PathBuf;

use rawdb::{Database, Region};

use crate::{AnyVec, Header, Result, Stamp, Version};

/// Trait for stored vectors that persist data to disk (as opposed to lazy computed vectors).
pub trait AnyStoredVec: AnyVec {
    fn db_path(&self) -> PathBuf;

    fn region(&self) -> &Region;

    fn header(&self) -> &Header;

    fn mut_header(&mut self) -> &mut Header;

    /// Number of stamped change files to keep for rollback support.
    fn saved_stamped_changes(&self) -> u16;

    /// Writes pending changes to storage.
    /// Returns `Ok(true)` if data was written, `Ok(false)` if nothing to write.
    /// This does not create a rollback checkpoint or synchronize files. Use
    /// stamped writes with changes before overwriting checkpointed values.
    /// After a failed or unwound write, discard the writer; retries and resets
    /// cannot repair a partially persisted logical vector.
    #[doc(hidden)]
    fn write(&mut self) -> Result<bool>;

    #[doc(hidden)]
    fn db(&self) -> Database;

    #[inline]
    fn flush(&mut self) -> Result<()> {
        let guard = self.header().begin_write()?;
        self.write()?;
        self.db().flush()?;
        guard.finish(Ok(()))
    }

    /// The actual length stored on disk.
    fn real_stored_len(&self) -> usize;
    /// The effective stored length (may differ from real_stored_len during truncation).
    fn stored_len(&self) -> usize;

    #[inline]
    fn update_stamp(&mut self, stamp: Stamp) {
        self.mut_header().update_stamp(stamp);
    }

    #[inline]
    fn stamp(&self) -> Stamp {
        self.header().stamp()
    }

    #[inline]
    fn stamped_write(&mut self, stamp: Stamp) -> Result<()> {
        let guard = self.header().begin_write()?;
        self.write()?;
        self.update_stamp(stamp);
        self.write()?;
        guard.finish(Ok(()))
    }

    /// Flushes with the given stamp, saving changes to enable rollback.
    /// Prefixed with `any_` to avoid conflict with `WritableVec::stamped_write_with_changes`.
    fn any_stamped_write_with_changes(&mut self, stamp: Stamp) -> Result<()>;

    /// Advances the in-memory rollback baseline to the current stored state.
    #[doc(hidden)]
    fn any_save_rollback_state(&mut self);

    /// Flushes with the given stamp, optionally saving changes for rollback.
    #[inline]
    fn any_stamped_write_maybe_with_changes(
        &mut self,
        stamp: Stamp,
        with_changes: bool,
    ) -> Result<()> {
        if with_changes {
            self.any_stamped_write_with_changes(stamp)
        } else {
            self.stamped_write(stamp)?;
            if self.saved_stamped_changes() > 0 {
                self.any_save_rollback_state();
            }
            Ok(())
        }
    }

    fn serialize_changes(&self) -> Result<Vec<u8>>;

    /// Removes this vector's region from the database.
    fn remove(self) -> Result<()>;

    /// Truncates the vector to the given length if it is longer.
    /// Prefixed with `any_` to avoid conflict with `WritableVec::truncate_if_needed_at`.
    fn any_truncate_if_needed_at(&mut self, index: usize) -> Result<()>;

    /// Resets the vector state, clearing all data.
    fn any_reset(&mut self) -> Result<()>;

    /// Includes this vector's version and clears data when dependencies change.
    fn any_validate_computed_version_or_reset(
        &mut self,
        dependency_version: Version,
    ) -> Result<()> {
        let version = self.header().vec_version() + dependency_version;
        if version != self.header().computed_version() {
            self.mut_header().update_computed_version(version);
            if !self.is_empty() {
                self.any_reset()?;
            }
        }
        Ok(())
    }
}
