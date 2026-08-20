use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AnyStoredVec, AnyVec, Bytes, BytesStrategy, BytesVec, BytesVecReader, BytesVecValue,
    ChangeCursor, ChangeData, ImportableVec, ReadWriteBaseVec, Stamp, StoredVec, ValueStrategy,
    VecIndex, WritableVec,
};

#[cfg(feature = "zerocopy")]
use crate::{VecReader, ZeroCopyStrategy, ZeroCopyVec, ZeroCopyVecValue};

use super::MutableVec;

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
        indices: &BTreeSet<usize>,
        previous: &BTreeMap<usize, Self::T>,
        bytes: &mut Vec<u8>,
    );
    fn parse_mutable_changes(
        bytes: &[u8],
    ) -> crate::Result<(Vec<(usize, Self::T)>, BTreeSet<usize>)>;
    fn save_change_file(&self, stamp: Stamp, bytes: &[u8]) -> crate::Result<()>;
    fn read_current_change_file(&self) -> crate::Result<Vec<u8>>;
    fn save_previous(&mut self);
    fn save_previous_for_rollback(&mut self);
}

macro_rules! impl_mutable_raw_vec {
    ($vec:ident, $value:ident, $strategy:ident, $reader:ty) => {
        impl<I, T> MutableRawVec for $vec<I, T>
        where
            I: VecIndex,
            T: $value,
        {
            type Reader = $reader;

            #[inline]
            fn reader(&self) -> Self::Reader {
                self.0.reader().into()
            }

            #[inline(always)]
            fn reader_len(reader: &Self::Reader) -> usize {
                reader.len()
            }

            #[inline(always)]
            fn read_stored(reader: &Self::Reader, index: usize) -> T {
                reader.get_at(index)
            }

            #[inline(always)]
            fn rollback_len(&self) -> usize {
                self.0.base.prev_stored_len()
            }

            #[inline]
            fn pushed_mut(&mut self) -> &mut Vec<T> {
                self.0.mut_pushed()
            }

            #[inline]
            fn reserve_pushed(&mut self, additional: usize) {
                self.0.reserve_pushed(additional);
            }

            fn write_updates(&mut self, updated: BTreeMap<usize, T>) {
                self.region().batch_write_ordered(
                    updated.into_iter().map(|(index, value)| {
                        (index * size_of::<T>() + crate::HEADER_OFFSET, value)
                    }),
                    size_of::<T>(),
                    $strategy::<T>::write_to_slice,
                );
            }

            fn append_previous_values(
                &self,
                indices: &BTreeSet<usize>,
                previous: &BTreeMap<usize, T>,
                bytes: &mut Vec<u8>,
            ) {
                let reader = self.reader();
                for index in indices {
                    let value = previous
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| reader.get_at(*index));
                    $strategy::<T>::write_to_vec(&value, bytes);
                }
            }

            fn parse_mutable_changes(
                bytes: &[u8],
            ) -> crate::Result<(Vec<(usize, T)>, BTreeSet<usize>)> {
                let mut cursor = ChangeCursor::new(bytes);
                let _: ChangeData<T> = ReadWriteBaseVec::<I, T>::parse_change_data(
                    &mut cursor,
                    size_of::<T>(),
                    $strategy::<T>::read,
                )?;
                let modified_len = cursor.read_u64()?;
                let indices =
                    cursor.read_values(modified_len, crate::SIZE_OF_U64, usize::from_bytes)?;
                let values =
                    cursor.read_values(modified_len, size_of::<T>(), $strategy::<T>::read)?;
                let previous_holes_len = cursor.read_u64()?;
                let previous_holes = cursor
                    .read_values(previous_holes_len, crate::SIZE_OF_U64, usize::from_bytes)?
                    .into_iter()
                    .collect();
                Ok((indices.into_iter().zip(values).collect(), previous_holes))
            }

            fn save_change_file(&self, stamp: Stamp, bytes: &[u8]) -> crate::Result<()> {
                self.0.base.save_change_file(stamp, bytes)
            }

            fn read_current_change_file(&self) -> crate::Result<Vec<u8>> {
                self.0.base.read_current_change_file()
            }

            fn save_previous(&mut self) {
                self.0.base.save_prev();
            }

            fn save_previous_for_rollback(&mut self) {
                self.0.base.save_prev_for_rollback();
            }
        }
    };
}

impl_mutable_raw_vec!(
    BytesVec,
    BytesVecValue,
    BytesStrategy,
    BytesVecReader<I, T>
);

#[cfg(feature = "zerocopy")]
impl_mutable_raw_vec!(
    ZeroCopyVec,
    ZeroCopyVecValue,
    ZeroCopyStrategy,
    VecReader<I, T, ZeroCopyStrategy<T>>
);

impl<V> MutableVec<V>
where
    V: MutableRawVec,
{
    #[inline]
    fn reader_inner(&self) -> V::Reader {
        self.vec.reader()
    }

    #[inline]
    fn get_with_reader_inner(&self, index: usize, reader: &V::Reader) -> Option<V::T> {
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
    fn delete_inner(&mut self, index: usize) {
        if index >= self.vec.len() {
            return;
        }
        if !self.current_updated().is_empty() {
            self.mut_updated().remove(&index);
        }
        self.mut_holes().insert(index);
    }

    fn collect_holed_inner(&self) -> Vec<Option<V::T>> {
        let reader = self.reader_inner();
        (0..self.vec.len())
            .map(|index| self.get_with_reader_inner(index, &reader))
            .collect()
    }

    fn take_inner(&mut self, index: usize, reader: &V::Reader) -> Option<V::T> {
        let value = self.get_with_reader_inner(index, reader);
        if value.is_some() {
            self.delete_inner(index);
        }
        value
    }

    fn fill_first_hole_or_push_inner(&mut self, value: V::T) -> crate::Result<V::I> {
        if let Some(index) = self.mut_holes().pop_first() {
            self.update_value_at(index, value)?;
            return Ok(V::I::from(index));
        }
        self.vec.push(value);
        Ok(V::I::from(self.vec.len() - 1))
    }
}

impl<I, T> MutableVec<BytesVec<I, T>>
where
    I: VecIndex,
    T: BytesVecValue,
{
    #[inline]
    pub fn reader(&self) -> BytesVecReader<I, T> {
        self.reader_inner()
    }

    #[inline]
    pub fn holes(&self) -> &BTreeSet<usize> {
        self.current_holes()
    }

    #[inline]
    pub fn prev_holes(&self) -> &BTreeSet<usize> {
        self.holes.previous()
    }

    #[inline]
    pub fn updated(&self) -> &BTreeMap<usize, T> {
        self.current_updated()
    }

    #[inline]
    pub fn prev_updated(&self) -> &BTreeMap<usize, T> {
        self.previous_updated()
    }

    pub fn collect_holed(&self) -> Vec<Option<T>> {
        self.collect_holed_inner()
    }

    #[inline]
    pub fn get_with_reader(&self, index: I, reader: &BytesVecReader<I, T>) -> Option<T> {
        self.get_with_reader_inner(index.to_usize(), reader)
    }

    #[inline]
    pub fn get_with_reader_at(&self, index: usize, reader: &BytesVecReader<I, T>) -> Option<T> {
        self.get_with_reader_inner(index, reader)
    }

    #[inline]
    pub fn get_append_only(&self, index: I, reader: &BytesVecReader<I, T>) -> Option<T> {
        debug_assert!(
            self.current_holes().is_empty() && self.current_updated().is_empty(),
            "get_append_only requires a vector without holes or updates"
        );
        self.vec.get_append_only(index, reader)
    }

    #[inline]
    pub fn update(&mut self, index: I, value: T) -> crate::Result<()> {
        self.update_value_at(index.to_usize(), value)
    }

    #[inline]
    pub fn update_at(&mut self, index: usize, value: T) -> crate::Result<()> {
        self.update_value_at(index, value)
    }

    #[inline]
    pub fn delete(&mut self, index: I) {
        self.delete_inner(index.to_usize());
    }

    #[inline]
    pub fn delete_at(&mut self, index: usize) {
        self.delete_inner(index);
    }

    #[inline]
    pub fn get_first_empty_index(&self) -> I {
        self.current_holes()
            .first()
            .copied()
            .map(I::from)
            .unwrap_or_else(|| I::from(self.vec.len()))
    }

    #[inline]
    pub fn fill_first_hole_or_push(&mut self, value: T) -> crate::Result<I> {
        self.fill_first_hole_or_push_inner(value)
    }

    #[inline]
    pub fn reserve_pushed(&mut self, additional: usize) {
        self.vec.reserve_pushed(additional);
    }

    pub fn take(&mut self, index: I, reader: &BytesVecReader<I, T>) -> Option<T> {
        self.take_inner(index.to_usize(), reader)
    }

    pub fn take_at(&mut self, index: usize, reader: &BytesVecReader<I, T>) -> Option<T> {
        self.take_inner(index, reader)
    }
}

#[cfg(feature = "zerocopy")]
impl<I, T> MutableVec<ZeroCopyVec<I, T>>
where
    I: VecIndex,
    T: ZeroCopyVecValue,
{
    #[inline]
    pub fn reader(&self) -> VecReader<I, T, ZeroCopyStrategy<T>> {
        self.reader_inner()
    }

    #[inline]
    pub fn holes(&self) -> &BTreeSet<usize> {
        self.current_holes()
    }

    #[inline]
    pub fn prev_holes(&self) -> &BTreeSet<usize> {
        self.holes.previous()
    }

    #[inline]
    pub fn updated(&self) -> &BTreeMap<usize, T> {
        self.current_updated()
    }

    #[inline]
    pub fn prev_updated(&self) -> &BTreeMap<usize, T> {
        self.previous_updated()
    }

    pub fn collect_holed(&self) -> Vec<Option<T>> {
        self.collect_holed_inner()
    }

    #[inline]
    pub fn get_with_reader(
        &self,
        index: I,
        reader: &VecReader<I, T, ZeroCopyStrategy<T>>,
    ) -> Option<T> {
        self.get_with_reader_inner(index.to_usize(), reader)
    }

    #[inline]
    pub fn get_with_reader_at(
        &self,
        index: usize,
        reader: &VecReader<I, T, ZeroCopyStrategy<T>>,
    ) -> Option<T> {
        self.get_with_reader_inner(index, reader)
    }

    #[inline]
    pub fn get_append_only(
        &self,
        index: I,
        reader: &VecReader<I, T, ZeroCopyStrategy<T>>,
    ) -> Option<T> {
        debug_assert!(
            self.current_holes().is_empty() && self.current_updated().is_empty(),
            "get_append_only requires a vector without holes or updates"
        );
        self.vec.get_append_only(index, reader)
    }

    #[inline]
    pub fn read_ref<'a>(
        &self,
        index: I,
        reader: &'a VecReader<I, T, ZeroCopyStrategy<T>>,
    ) -> Option<&'a T> {
        let index = index.to_usize();
        if self.current_holes().contains(&index) || self.current_updated().contains_key(&index) {
            return None;
        }
        self.vec.read_ref_at(index, reader)
    }

    #[inline]
    pub fn update(&mut self, index: I, value: T) -> crate::Result<()> {
        self.update_value_at(index.to_usize(), value)
    }

    #[inline]
    pub fn update_at(&mut self, index: usize, value: T) -> crate::Result<()> {
        self.update_value_at(index, value)
    }

    #[inline]
    pub fn delete(&mut self, index: I) {
        self.delete_inner(index.to_usize());
    }

    #[inline]
    pub fn delete_at(&mut self, index: usize) {
        self.delete_inner(index);
    }

    #[inline]
    pub fn get_first_empty_index(&self) -> I {
        self.current_holes()
            .first()
            .copied()
            .map(I::from)
            .unwrap_or_else(|| I::from(self.vec.len()))
    }

    #[inline]
    pub fn fill_first_hole_or_push(&mut self, value: T) -> crate::Result<I> {
        self.fill_first_hole_or_push_inner(value)
    }

    #[inline]
    pub fn reserve_pushed(&mut self, additional: usize) {
        self.vec.reserve_pushed(additional);
    }

    pub fn take(&mut self, index: I, reader: &VecReader<I, T, ZeroCopyStrategy<T>>) -> Option<T> {
        self.take_inner(index.to_usize(), reader)
    }

    pub fn take_at(
        &mut self,
        index: usize,
        reader: &VecReader<I, T, ZeroCopyStrategy<T>>,
    ) -> Option<T> {
        self.take_inner(index, reader)
    }
}
