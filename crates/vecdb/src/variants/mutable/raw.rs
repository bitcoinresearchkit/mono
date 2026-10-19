use std::collections::{BTreeMap, BTreeSet};

use super::MutableVec;
use crate::{AnyStoredVec, ChangeData, ImportOptions, Result, Stamp, StoredVec, VecIndex};

pub trait MutableRawVec: StoredVec {
    type Reader;

    /// Imports without resetting: a version or format mismatch is an error, so the
    /// wrapper can reset its own regions along with the vector.
    fn import_strict_with(options: ImportOptions) -> Result<Self>;

    fn reader(&self) -> Self::Reader;
    fn reader_len(reader: &Self::Reader) -> usize;
    fn read_stored(reader: &Self::Reader, index: usize) -> Self::T;
    fn rollback_len(&self) -> usize;
    /// See [`crate::ReadWriteBaseVec::unwritten_baseline`].
    fn unwritten_baseline(&self) -> bool;
    fn pushed_mut(&mut self) -> &mut Vec<Self::T>;
    fn reserve_pushed(&mut self, additional: usize);
    fn write_updates(&mut self, updated: BTreeMap<usize, Self::T>) -> Result<()>;
    fn append_previous_values(
        &self,
        indices: &[usize],
        previous: &BTreeMap<usize, Self::T>,
        bytes: &mut Vec<u8>,
    );
    /// The base change record, the previous values of updated indexes and the previous holes.
    fn parse_mutable_changes(
        bytes: &[u8],
    ) -> Result<(ChangeData<Self::T>, Vec<(usize, Self::T)>, BTreeSet<usize>)>;
    /// Undoes the base change record of [`Self::parse_mutable_changes`].
    fn rollback_change(&mut self, change: ChangeData<Self::T>) -> Result<()>;
    fn save_change_file(&self, stamp: Stamp, bytes: &[u8]) -> Result<()>;
    fn read_current_change_file(&self) -> Result<Vec<u8>>;
    fn save_previous(&mut self);
    fn save_previous_for_rollback(&mut self);
}

impl<V> MutableVec<V>
where
    V: MutableRawVec,
{
    #[inline]
    pub fn reader(&self) -> V::Reader {
        self.vec.reader()
    }

    #[inline]
    pub(crate) fn get_with_reader_at(&self, index: usize, reader: &V::Reader) -> Option<V::T> {
        if self.holes().contains(&index) {
            return None;
        }

        let stored_len = V::reader_len(reader);
        debug_assert_eq!(stored_len, self.vec.stored_len(), "stale VecReader");
        if index >= stored_len {
            return self.vec.pushed().get(index - stored_len).cloned();
        }

        if let Some(value) = self.updated.current.get(&index) {
            return Some(value.clone());
        }

        Some(V::read_stored(reader, index))
    }

    #[inline]
    pub(crate) fn delete_at(&mut self, index: usize) {
        self.header().assert_writable();
        if index >= self.vec.len() {
            return;
        }
        self.updated.current.remove(&index);
        self.mut_holes().insert(index);
    }

    #[inline]
    pub(crate) fn fill_first_hole_or_push(&mut self, value: V::T) -> Result<V::I> {
        self.header().check_writable()?;
        if let Some(index) = self.mut_holes().pop_first() {
            self.update_at(index, value)?;
            return Ok(V::I::from(index));
        }
        self.vec.push(value);
        Ok(V::I::from(self.vec.len() - 1))
    }

    #[inline]
    pub fn get_with_reader(&self, index: V::I, reader: &V::Reader) -> Option<V::T> {
        self.get_with_reader_at(index.to_usize(), reader)
    }

    /// Borrows the staged values for mutation without changing their length.
    #[inline]
    pub fn pushed_mut(&mut self) -> &mut [V::T] {
        self.vec.pushed_mut()
    }
}
