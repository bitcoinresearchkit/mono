use std::path::PathBuf;

use rawdb::{Database, Region};

use crate::{AnyVec, Header, Result, Stamp, Version};

pub trait AnyStoredVec: AnyVec {
    fn db_path(&self) -> PathBuf;

    fn region(&self) -> &Region;

    fn header(&self) -> &Header;

    fn mut_header(&mut self) -> &mut Header;

    /// Number of stamped change files to keep for rollback support.
    fn saved_stamped_changes(&self) -> u16;

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
        self.db().flush();
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

    /// Persists a changed header, such as a new stamp, without touching data.
    #[doc(hidden)]
    fn write_header(&mut self) -> Result<()> {
        if self.header().modified() {
            let region = self.region().clone();
            self.mut_header().write(&region)?;
        }
        Ok(())
    }

    /// Writes the data, then the header with its new stamp.
    #[inline]
    fn stamped_write(&mut self, stamp: Stamp) -> Result<()> {
        let guard = self.header().begin_write()?;
        self.write()?;
        self.update_stamp(stamp);
        self.write_header()?;
        guard.finish(Ok(()))
    }

    /// Saves the undo record at `stamp` of everything since the last checkpoint. Only called
    /// with saved stamped changes, before the checkpoint's write overwrites anything.
    #[doc(hidden)]
    fn any_save_changes(&self, stamp: Stamp) -> Result<()>;

    /// Writes the checkpoint's data, stamp and header without reading back values for undo
    /// records, then (with saved stamped changes) advances the rollback baseline to it.
    #[doc(hidden)]
    fn any_write_checkpoint(&mut self, stamp: Stamp) -> Result<()>;

    /// Removes the change files above `stamp`, the stamp a write without undo starts from.
    #[doc(hidden)]
    fn any_remove_changes_above(&self, stamp: Stamp) -> Result<()>;

    /// Writes a checkpoint at `stamp`. With changes (and saved stamped changes), its undo record
    /// is saved first; without, the change files above the current stamp, which undo states the
    /// write replaces, are removed. Either way the rollback baseline advances to the written state.
    fn any_stamped_write_maybe_with_changes(
        &mut self,
        stamp: Stamp,
        with_changes: bool,
    ) -> Result<()> {
        if !with_changes {
            self.any_remove_changes_above(self.stamp())?;
        }
        let guard = self.header().begin_write()?;
        if with_changes && self.saved_stamped_changes() > 0 {
            self.any_save_changes(stamp)?;
        }
        self.any_write_checkpoint(stamp)?;
        guard.finish(Ok(()))
    }

    fn serialize_changes(&self) -> Result<Vec<u8>>;

    /// Removes this vector's storage regions from the database.
    fn remove(self) -> Result<()>;

    /// Prefixed with `any_` to avoid conflict with `WritableVec::truncate_if_needed_at`.
    fn any_truncate_if_needed_at(&mut self, index: usize) -> Result<()>;

    /// Resets the vector state, clearing all data.
    fn any_reset(&mut self) -> Result<()>;

    /// Includes this vector's version and clears data when dependencies change.
    /// Returns whether the computed version changed.
    fn any_validate_computed_version_or_reset(
        &mut self,
        dependency_version: Version,
    ) -> Result<bool> {
        let version = self.header().vec_version() + dependency_version;
        if version == self.header().computed_version() {
            return Ok(false);
        }
        self.mut_header().update_computed_version(version);
        // Even when it looks empty: a pending truncation hides rows still on disk, and only the
        // reset's empty baseline keeps a cancel from bringing them back under the new version.
        self.any_reset()?;
        Ok(true)
    }
}
