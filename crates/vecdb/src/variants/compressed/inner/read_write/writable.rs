use std::{collections::BTreeMap, path::PathBuf};

use crate::{AnyStoredVec, Stamp, VecIndex, VecValue, WritableVec};

use super::{super::CompressionStrategy, ReadWriteCompressedVec};

impl<I, T, S> WritableVec<I, T> for ReadWriteCompressedVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    #[inline]
    fn push(&mut self, value: T) {
        self.base.mut_pushed().push(value);
    }

    #[inline]
    fn pushed(&self) -> &[T] {
        self.base.pushed()
    }

    fn truncate_if_needed_at(&mut self, index: usize) -> crate::Result<()> {
        if self.base.truncate_pushed(index) {
            self.base.update_stored_len(index);
        }
        Ok(())
    }

    fn reset(&mut self) -> crate::Result<()> {
        self.pages.write().reset();
        self.truncate_if_needed_at(0)?;
        self.base.reset_base()
    }

    fn reset_unsaved(&mut self) {
        self.base.reset_unsaved_base();
    }

    fn is_dirty(&self) -> bool {
        !self.base.pushed().is_empty()
    }

    fn stamped_write_with_changes(&mut self, stamp: Stamp) -> crate::Result<()> {
        if self.base.saved_stamped_changes() == 0 {
            return self.stamped_write(stamp);
        }

        let data = self.serialize_changes()?;
        self.base.save_change_file(stamp, &data)?;
        self.stamped_write(stamp)?;
        self.base.save_prev();

        Ok(())
    }

    fn rollback(&mut self) -> crate::Result<()> {
        let bytes = self.base.read_current_change_file()?;
        self.deserialize_then_undo_changes(&bytes)
    }

    fn find_rollback_files(&self) -> crate::Result<BTreeMap<Stamp, PathBuf>> {
        self.base.find_rollback_files()
    }

    fn save_rollback_state(&mut self) {
        self.base.save_prev_for_rollback();
    }
}
