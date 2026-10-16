use std::path::PathBuf;

use rawdb::{Database, Region};

use super::{MutableRawVec, MutableVec};
use crate::{AnyStoredVec, Bytes, Header, Result, Stamp, WritableVec};

impl<V> AnyStoredVec for MutableVec<V>
where
    V: MutableRawVec,
{
    #[inline]
    fn db_path(&self) -> PathBuf {
        self.vec.db_path()
    }

    #[inline]
    fn region(&self) -> &Region {
        self.vec.region()
    }

    #[inline]
    fn header(&self) -> &Header {
        self.vec.header()
    }

    #[inline]
    fn mut_header(&mut self) -> &mut Header {
        self.vec.mut_header()
    }

    #[inline]
    fn saved_stamped_changes(&self) -> u16 {
        self.vec.saved_stamped_changes()
    }

    fn write(&mut self) -> Result<bool> {
        self.write_inner(true)
    }

    #[inline]
    fn db(&self) -> Database {
        self.vec.db()
    }

    #[inline]
    fn real_stored_len(&self) -> usize {
        self.vec.real_stored_len()
    }

    #[inline]
    fn stored_len(&self) -> usize {
        self.vec.stored_len()
    }

    fn any_save_changes(&self, stamp: Stamp) -> Result<()> {
        self.vec.save_change_file(stamp, &self.serialize_changes()?)
    }

    /// The undo record is saved or discarded already: the updated values' previous ones are not
    /// read back.
    fn any_write_checkpoint(&mut self, stamp: Stamp) -> Result<()> {
        let guard = self.header().begin_write()?;
        self.write_inner(false)?;
        self.update_stamp(stamp);
        self.write_header()?;
        if self.saved_stamped_changes() > 0 {
            self.vec.save_previous();
            self.holes.save();
            self.updated.clear_previous();
        }
        guard.finish(Ok(()))
    }

    fn any_remove_changes_above(&self, stamp: Stamp) -> Result<()> {
        self.vec.any_remove_changes_above(stamp)
    }

    fn serialize_changes(&self) -> Result<Vec<u8>> {
        let mut bytes = self.vec.serialize_changes()?;
        let rollback_len = self.vec.rollback_len();
        let mut indices = self
            .current_updated()
            .keys()
            .chain(self.previous_updated().keys())
            .filter(|&&index| index < rollback_len)
            .copied()
            .collect::<Vec<_>>();
        indices.sort_unstable();
        indices.dedup();

        let previous_holes = self.holes.previous();
        bytes.reserve(
            indices.len() * (size_of::<usize>() + size_of::<V::T>())
                + (previous_holes.len() + 2) * size_of::<usize>(),
        );

        bytes.extend(indices.len().to_bytes());
        for index in &indices {
            bytes.extend(index.to_bytes());
        }
        self.vec
            .append_previous_values(&indices, self.previous_updated(), &mut bytes);

        bytes.extend(previous_holes.len().to_bytes());
        for hole in previous_holes.iter() {
            bytes.extend(hole.to_bytes());
        }

        Ok(bytes)
    }

    fn remove(self) -> Result<()> {
        let db = self.vec.db();
        let holes_region_name = self.holes_region_name();
        let has_stored_holes = self.has_stored_holes;
        self.vec.remove()?;
        if has_stored_holes {
            db.remove_region(&holes_region_name)?;
        }
        Ok(())
    }

    fn any_truncate_if_needed_at(&mut self, index: usize) -> Result<()> {
        <Self as WritableVec<V::I, V::T>>::truncate_if_needed_at(self, index)
    }

    fn any_reset(&mut self) -> Result<()> {
        <Self as WritableVec<V::I, V::T>>::reset(self)
    }
}
