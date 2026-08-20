use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use crate::{AnyStoredVec, Stamp, WritableVec};

use super::{MutableRawVec, MutableVec};

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

    fn truncate_if_needed_at(&mut self, index: usize) -> crate::Result<()> {
        self.truncate_mutations_at(index);
        self.vec.truncate_if_needed_at(index)
    }

    fn reset(&mut self) -> crate::Result<()> {
        self.holes.clear();
        self.updated.clear();
        self.vec.reset()?;
        Ok(())
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

    fn stamped_write_with_changes(&mut self, stamp: Stamp) -> crate::Result<()> {
        if self.vec.saved_stamped_changes() == 0 {
            return self.stamped_write(stamp);
        }

        let bytes = self.serialize_changes()?;
        self.vec.save_change_file(stamp, &bytes)?;
        self.update_stamp(stamp);
        self.write_inner(false)?;
        self.vec.save_previous();
        self.holes.save();
        self.updated.clear_previous();
        Ok(())
    }

    fn rollback(&mut self) -> crate::Result<()> {
        let bytes = self.vec.read_current_change_file()?;
        let (modifications, previous_holes) = V::parse_mutable_changes(&bytes)?;

        self.vec.rollback()?;
        self.truncate_mutations_at(self.vec.len());
        for (index, value) in modifications {
            self.update_value_at(index, value)?;
        }
        *self.holes.current_mut() = Arc::new(previous_holes);
        self.holes.save();
        self.updated.save();
        Ok(())
    }

    fn find_rollback_files(&self) -> crate::Result<BTreeMap<Stamp, PathBuf>> {
        self.vec.find_rollback_files()
    }

    fn save_rollback_state(&mut self) {
        self.vec.save_previous_for_rollback();
        self.holes.save();
        self.updated.save();
    }
}
