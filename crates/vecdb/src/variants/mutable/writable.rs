use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use super::{MutableRawVec, MutableVec};
use crate::{AnyStoredVec, Result, Stamp, WritableVec};

impl<V: MutableRawVec> Extend<V::T> for MutableVec<V> {
    fn extend<T: IntoIterator<Item = V::T>>(&mut self, values: T) {
        self.vec.pushed_mut().extend(values);
    }
}

impl<V> WritableVec<V::I, V::T> for MutableVec<V>
where
    V: MutableRawVec,
{
    #[inline]
    fn push(&mut self, value: V::T) {
        self.vec.push(value)
    }

    #[inline]
    fn pushed(&self) -> &[V::T] {
        self.vec.pushed()
    }

    fn truncate_if_needed_at(&mut self, index: usize) -> Result<()> {
        self.header().check_writable()?;
        self.truncate_mutations_at(index);
        self.vec.truncate_if_needed_at(index)
    }

    fn reset(&mut self) -> Result<()> {
        let guard = self.header().begin_write()?;
        self.holes.clear();
        self.updated.clear();
        self.vec.reset()?;
        guard.finish(Ok(()))
    }

    fn reset_unsaved(&mut self) {
        self.vec.reset_unsaved();
        *self.holes.current_mut() = Arc::clone(&self.published_holes.read());
        self.updated.current_mut().clear();
    }

    #[inline]
    fn is_dirty(&self) -> bool {
        self.vec.is_dirty() || self.holes_changed() || !self.current_updated().is_empty()
    }

    fn stamped_write_with_changes(&mut self, stamp: Stamp) -> Result<()> {
        let guard = self.header().begin_write()?;
        self.save_changes(stamp)?;
        self.write_saved_changes(stamp)?;
        guard.finish(Ok(()))
    }

    fn rollback(&mut self) -> Result<()> {
        self.header().check_writable()?;
        let bytes = self.vec.read_current_change_file()?;
        let (modifications, previous_holes) = V::parse_mutable_changes(&bytes)?;
        let guard = self.header().begin_write()?;

        // Undo from the last save point, like the wrapped vec: unsaved updates are dropped (the
        // saved map holds what earlier undos restored and the stamp's values of rows written
        // since). Holes need no reset: the undo replaces them below.
        *self.updated.current_mut() = self.updated.previous().clone();
        self.vec.rollback()?;
        // Restored rows past the stored length live in the pushed values: updates there are stale.
        self.truncate_mutations_at(self.vec.stored_len());
        for (index, value) in modifications {
            self.update_value_at(index, value)?;
        }
        *self.holes.current_mut() = Arc::new(previous_holes);
        // The modifications may land in restored pushed rows: the next undo starts from them.
        self.vec.save_previous_for_rollback();
        self.holes.save();
        self.updated.save();
        guard.finish(Ok(()))
    }

    fn find_rollback_files(&self) -> Result<BTreeMap<Stamp, PathBuf>> {
        self.vec.find_rollback_files()
    }

    fn save_rollback_state(&mut self) {
        self.header().assert_writable();
        self.vec.save_previous_for_rollback();
        self.holes.save();
        self.updated.save();
    }
}
