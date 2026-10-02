use std::collections::{BTreeMap, BTreeSet};

use super::MutableVec;
use crate::{AnyStoredVec, ImportableVec, Result, Stamp, StoredVec, VecIndex, WritableVec};

pub trait MutableRawVec: StoredVec + ImportableVec + WritableVec<Self::I, Self::T> + Sized {
    type Reader;

    fn reader(&self) -> Self::Reader;
    fn reader_len(reader: &Self::Reader) -> usize;
    fn read_stored(reader: &Self::Reader, index: usize) -> Self::T;
    fn rollback_len(&self) -> usize;
    fn pushed_mut(&mut self) -> &mut Vec<Self::T>;
    fn reserve_pushed(&mut self, additional: usize);
    fn write_updates(&mut self, updated: BTreeMap<usize, Self::T>);
    fn append_previous_values(
        &self,
        indices: &[usize],
        previous: &BTreeMap<usize, Self::T>,
        bytes: &mut Vec<u8>,
    );
    fn parse_mutable_changes(bytes: &[u8]) -> Result<(Vec<(usize, Self::T)>, BTreeSet<usize>)>;
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
    pub fn get_with_reader_at(&self, index: usize, reader: &V::Reader) -> Option<V::T> {
        if !self.current_holes().is_empty() && self.current_holes().contains(&index) {
            return None;
        }

        let stored_len = V::reader_len(reader);
        debug_assert_eq!(stored_len, self.vec.stored_len(), "stale VecReader");
        if index >= stored_len {
            return self.vec.pushed().get(index - stored_len).cloned();
        }

        if !self.current_updated().is_empty()
            && let Some(value) = self.current_updated().get(&index)
        {
            return Some(value.clone());
        }

        Some(V::read_stored(reader, index))
    }

    #[inline]
    pub fn delete_at(&mut self, index: usize) {
        self.header().assert_writable();
        if index >= self.vec.len() {
            return;
        }
        if !self.current_updated().is_empty() {
            self.mut_updated().remove(&index);
        }
        self.mut_holes().insert(index);
    }

    pub fn collect_holed(&self) -> Vec<Option<V::T>> {
        let reader = self.reader();
        (0..self.vec.len())
            .map(|index| self.get_with_reader_at(index, &reader))
            .collect()
    }

    pub fn take_at(&mut self, index: usize, reader: &V::Reader) -> Option<V::T> {
        let value = self.get_with_reader_at(index, reader);
        if value.is_some() {
            self.delete_at(index);
        }
        value
    }

    #[inline]
    pub fn fill_first_hole_or_push(&mut self, value: V::T) -> Result<V::I> {
        self.header().check_writable()?;
        if let Some(index) = self.mut_holes().pop_first() {
            self.update_value_at(index, value)?;
            return Ok(V::I::from(index));
        }
        self.vec.push(value);
        Ok(V::I::from(self.vec.len() - 1))
    }

    #[inline]
    pub fn holes(&self) -> &BTreeSet<usize> {
        self.current_holes()
    }

    #[inline]
    pub fn get_with_reader(&self, index: V::I, reader: &V::Reader) -> Option<V::T> {
        self.get_with_reader_at(index.to_usize(), reader)
    }

    #[inline]
    pub fn update(&mut self, index: V::I, value: V::T) -> Result<()> {
        self.update_value_at(index.to_usize(), value)
    }

    #[inline]
    pub fn update_at(&mut self, index: usize, value: V::T) -> Result<()> {
        self.update_value_at(index, value)
    }

    #[inline]
    pub fn delete(&mut self, index: V::I) {
        self.delete_at(index.to_usize());
    }

    /// Borrows the staged values for mutation without changing their length.
    #[inline]
    pub fn pushed_mut(&mut self) -> &mut [V::T] {
        self.vec.pushed_mut()
    }

    pub fn take(&mut self, index: V::I, reader: &V::Reader) -> Option<V::T> {
        self.take_at(index.to_usize(), reader)
    }
}
