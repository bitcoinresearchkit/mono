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
        if self.vec.unwritten_baseline() {
            // Back to what the rollback restored (saved as the undo baseline), or to a reset's
            // empty state.
            self.holes.restore();
            self.updated.restore();
        } else {
            self.holes.current = Arc::clone(&self.published_holes.read());
            self.updated.current.clear();
        }
        self.vec.reset_unsaved();
    }

    #[inline]
    fn is_dirty(&self) -> bool {
        self.vec.is_dirty() || self.holes_changed() || !self.updated.current.is_empty()
    }

    fn rollback(&mut self) -> Result<()> {
        self.header().check_writable()?;
        let bytes = self.vec.read_current_change_file()?;
        let (change, modifications, previous_holes) = V::parse_mutable_changes(&bytes)?;
        let guard = self.header().begin_write()?;

        // Undo from the last save point, like the wrapped vec: unsaved updates are dropped (the
        // saved map holds what earlier undos restored and the stamp's values of rows written
        // since). Holes need no reset: the undo replaces them below.
        self.updated.restore();
        self.vec.rollback_change(change)?;
        // Restored rows past the stored length live in the pushed values: updates there are stale.
        self.truncate_mutations_at(self.vec.stored_len());
        for (index, value) in modifications {
            self.update_at(index, value)?;
        }
        self.holes.current = Arc::new(previous_holes);
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
