use std::path::PathBuf;

use rawdb::{Database, Region};

use super::EagerVec;
use crate::{AnyStoredVec, Header, Result, Stamp, StoredVec, WritableVec};

impl<V> AnyStoredVec for EagerVec<V>
where
    V: StoredVec,
{
    #[inline]
    fn db_path(&self) -> PathBuf {
        self.0.db_path()
    }

    #[inline]
    fn region(&self) -> &Region {
        self.0.region()
    }

    #[inline]
    fn header(&self) -> &Header {
        self.0.header()
    }

    #[inline]
    fn mut_header(&mut self) -> &mut Header {
        self.0.mut_header()
    }

    #[inline]
    fn saved_stamped_changes(&self) -> u16 {
        self.0.saved_stamped_changes()
    }

    #[inline]
    fn write(&mut self) -> Result<bool> {
        self.0.write()
    }

    #[inline]
    fn stored_len(&self) -> usize {
        self.0.stored_len()
    }

    #[inline]
    fn real_stored_len(&self) -> usize {
        self.0.real_stored_len()
    }

    #[inline]
    fn serialize_changes(&self) -> Result<Vec<u8>> {
        self.0.serialize_changes()
    }

    #[inline]
    fn db(&self) -> Database {
        self.0.db()
    }

    fn any_stamped_write_with_changes(&mut self, stamp: Stamp) -> Result<()> {
        self.0.stamped_write_with_changes(stamp)
    }

    fn any_save_rollback_state(&mut self) {
        self.0.save_rollback_state()
    }

    fn any_remove_changes_above(&self, stamp: Stamp) -> Result<()> {
        self.0.any_remove_changes_above(stamp)
    }

    fn remove(self) -> Result<()> {
        self.0.remove()
    }

    fn any_truncate_if_needed_at(&mut self, index: usize) -> Result<()> {
        self.truncate_if_needed_at(index)
    }

    fn any_reset(&mut self) -> Result<()> {
        self.reset()
    }
}
