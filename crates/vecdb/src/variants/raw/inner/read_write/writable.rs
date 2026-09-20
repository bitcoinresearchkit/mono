use std::{collections::BTreeMap, path::PathBuf};

use super::{super::RawStrategy, ReadWriteRawVec};
use crate::{
    AnyStoredVec, ChangeCursor, ReadWriteBaseVec, Result, Stamp, VecIndex, VecValue, WritableVec,
    cache::CachePolicy,
};

impl<I, T, S, C: CachePolicy> WritableVec<I, T> for ReadWriteRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    #[inline]
    fn push(&mut self, value: T) {
        self.base.mut_pushed().push(value);
    }

    #[inline]
    fn pushed(&self) -> &[T] {
        self.base.pushed()
    }

    fn truncate_if_needed_at(&mut self, index: usize) -> Result<()> {
        self.header().check_writable()?;
        self.with_cache_update(index, |this| {
            if this.base.truncate_pushed(index) {
                this.base.update_stored_len(index);
            }
            Ok(())
        })
    }

    fn reset(&mut self) -> Result<()> {
        let guard = self.header().begin_write()?;
        let result = self.with_cache_update(0, |this| {
            this.base.truncate_pushed(0);
            this.base.update_stored_len(0);
            this.base.reset_base()
        });
        guard.finish(result)
    }

    fn reset_unsaved(&mut self) {
        self.with_cache_update(0, |this| {
            this.base.reset_unsaved_base();
            Ok(())
        })
        .expect("resetting unsaved state cannot fail");
    }

    fn is_dirty(&self) -> bool {
        !self.base.pushed().is_empty()
    }

    fn stamped_write_with_changes(&mut self, stamp: Stamp) -> Result<()> {
        self.header().check_writable()?;
        if self.base.saved_stamped_changes() == 0 {
            return self.stamped_write(stamp);
        }

        let guard = self.header().begin_write()?;
        let data = self.serialize_changes()?;
        self.base.save_change_file(stamp, &data)?;
        self.stamped_write(stamp)?;
        self.base.save_prev();

        guard.finish(Ok(()))
    }

    fn rollback(&mut self) -> Result<()> {
        self.header().check_writable()?;
        let bytes = self.base.read_current_change_file()?;
        let change =
            ReadWriteBaseVec::<I, T>::parse_change_data::<S>(&mut ChangeCursor::new(&bytes))?;
        let (stamp, stored_len, pushed) = change.into_rollback(|| self.real_stored_len());
        self.with_cache_update(stored_len, |this| {
            this.base.apply_rollback(stamp, stored_len, pushed);
            Ok(())
        })
    }

    fn find_rollback_files(&self) -> Result<BTreeMap<Stamp, PathBuf>> {
        self.base.find_rollback_files()
    }

    fn save_rollback_state(&mut self) {
        self.header().assert_writable();
        self.base.save_prev_for_rollback();
    }
}
