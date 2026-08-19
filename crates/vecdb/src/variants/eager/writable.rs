use std::{collections::BTreeMap, path::PathBuf};

use crate::{Stamp, StoredVec, WritableVec};

use super::EagerVec;

impl<V> WritableVec<V::I, V::T> for EagerVec<V>
where
    V: StoredVec,
{
    #[inline]
    fn push(&mut self, value: V::T) {
        self.0.push(value);
    }

    #[inline]
    fn pushed(&self) -> &[V::T] {
        self.0.pushed()
    }

    #[inline]
    fn truncate_if_needed_at(&mut self, index: usize) -> crate::Result<()> {
        self.0.truncate_if_needed_at(index)
    }

    #[inline]
    fn reset(&mut self) -> crate::Result<()> {
        self.0.reset()
    }

    #[inline]
    fn reset_unsaved(&mut self) {
        self.0.reset_unsaved()
    }

    #[inline]
    fn is_dirty(&self) -> bool {
        self.0.is_dirty()
    }

    #[inline]
    fn stamped_write_with_changes(&mut self, stamp: Stamp) -> crate::Result<()> {
        self.0.stamped_write_with_changes(stamp)
    }

    #[inline]
    fn rollback(&mut self) -> crate::Result<()> {
        self.0.rollback()
    }

    fn find_rollback_files(&self) -> crate::Result<BTreeMap<Stamp, PathBuf>> {
        self.0.find_rollback_files()
    }

    fn save_rollback_state(&mut self) {
        self.0.save_rollback_state()
    }
}
