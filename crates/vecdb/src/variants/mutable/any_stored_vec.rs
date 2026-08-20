use std::{collections::BTreeSet, path::PathBuf};

use rawdb::{Database, Region};

use crate::{AnyStoredVec, Bytes, Header, Stamp, WritableVec};

use super::{MutableRawVec, MutableVec};

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

    fn write(&mut self) -> crate::Result<bool> {
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

    fn any_stamped_write_with_changes(&mut self, stamp: Stamp) -> crate::Result<()> {
        <Self as WritableVec<V::I, V::T>>::stamped_write_with_changes(self, stamp)
    }

    fn any_save_rollback_state(&mut self) {
        <Self as WritableVec<V::I, V::T>>::save_rollback_state(self)
    }

    fn serialize_changes(&self) -> crate::Result<Vec<u8>> {
        let mut bytes = self.vec.serialize_changes()?;
        let rollback_len = self.vec.rollback_len();
        let indices = self
            .current_updated()
            .keys()
            .chain(self.previous_updated().keys())
            .filter(|&&index| index < rollback_len)
            .copied()
            .collect::<BTreeSet<_>>();

        bytes.extend(indices.len().to_bytes());
        for index in &indices {
            bytes.extend(index.to_bytes());
        }
        self.vec
            .append_previous_values(&indices, self.previous_updated(), &mut bytes);

        let previous_holes = self.holes.previous();
        bytes.extend(previous_holes.len().to_bytes());
        for hole in previous_holes.iter() {
            bytes.extend(hole.to_bytes());
        }

        Ok(bytes)
    }

    fn remove(self) -> crate::Result<()> {
        let db = self.vec.db();
        let holes_region_name = self.holes_region_name();
        let has_stored_holes = self.has_stored_holes;
        self.vec.remove()?;
        if has_stored_holes {
            db.remove_region(&holes_region_name)?;
        }
        Ok(())
    }

    fn any_truncate_if_needed_at(&mut self, index: usize) -> crate::Result<()> {
        <Self as WritableVec<V::I, V::T>>::truncate_if_needed_at(self, index)
    }

    fn any_reset(&mut self) -> crate::Result<()> {
        <Self as WritableVec<V::I, V::T>>::reset(self)
    }
}
