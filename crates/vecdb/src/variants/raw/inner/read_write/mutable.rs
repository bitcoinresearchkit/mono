//! In-place updates and their undo records, for the `MutableVec` wrapping this vec.

use std::collections::{BTreeMap, BTreeSet};

use super::{super::RawStrategy, ReadWriteRawVec};
use crate::{
    AnyStoredVec, Bytes, ChangeCursor, ChangeData, HEADER_OFFSET, ImportOptions, ReadWriteBaseVec,
    Result, SIZE_OF_U64, Stamp, VecIndex, VecReader, VecValue,
    variants::mutable::raw::MutableRawVec,
};

impl<I, T, S> MutableRawVec for ReadWriteRawVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    type Reader = VecReader<I, T, S>;

    fn import_strict_with(options: ImportOptions) -> Result<Self> {
        ReadWriteRawVec::import_strict_with(options)
    }

    #[inline]
    fn reader(&self) -> Self::Reader {
        VecReader::from_read_write(self)
    }

    #[inline(always)]
    fn reader_len(reader: &Self::Reader) -> usize {
        reader.stored_len()
    }

    #[inline(always)]
    fn read_stored(reader: &Self::Reader, index: usize) -> T {
        reader.get_at(index)
    }

    #[inline(always)]
    fn rollback_len(&self) -> usize {
        self.base.previous_stored_len
    }

    #[inline]
    fn unwritten_baseline(&self) -> bool {
        self.base.unwritten_baseline
    }

    #[inline]
    fn pushed_mut(&mut self) -> &mut Vec<T> {
        self.base.mut_pushed()
    }

    #[inline]
    fn reserve_pushed(&mut self, additional: usize) {
        self.base.mut_pushed().reserve(additional);
    }

    fn write_updates(&mut self, updated: BTreeMap<usize, T>) -> Result<()> {
        let guard = self.header().begin_write()?;
        let region = self.region();
        let result = if self.base.unread_updates {
            // SAFETY: the importer vouched for its readers (`ImportOptions::with_unread_updates`),
            // whose one-value reads borrow only that value.
            unsafe {
                region.write_indexed_unread(
                    updated,
                    Self::SIZE_OF_T,
                    HEADER_OFFSET,
                    S::write_to_slice,
                )
            }
        } else {
            region.write_indexed(updated, Self::SIZE_OF_T, HEADER_OFFSET, S::write_to_slice)
        }
        .map_err(Into::into);
        guard.finish(result)
    }

    /// Serializes the value each index held before its pending update: from `previous` when
    /// it was already updated, else from storage.
    fn append_previous_values(
        &self,
        indices: &[usize],
        previous: &BTreeMap<usize, T>,
        bytes: &mut Vec<u8>,
    ) {
        let reader = VecReader::<I, T, S>::from_read_write(self);
        for &index in indices {
            match previous.get(&index) {
                Some(value) => S::write_to_vec(value, bytes),
                None => S::write_to_vec(&reader.get_at(index), bytes),
            }
        }
    }

    /// Parses a mutable change file: the base change data, then the previous values of updated
    /// indexes, then the previous holes.
    fn parse_mutable_changes(
        bytes: &[u8],
    ) -> Result<(ChangeData<T>, Vec<(usize, T)>, BTreeSet<usize>)> {
        let mut cursor = ChangeCursor::new(bytes);
        let change = ReadWriteBaseVec::<I, T>::parse_change_data::<S>(&mut cursor)?;
        let modified_len = cursor.read_u64()?;
        let indices = cursor.read_values(modified_len, SIZE_OF_U64, usize::from_bytes)?;
        let values = cursor.read_values(modified_len, Self::SIZE_OF_T, S::read)?;
        let previous_holes_len = cursor.read_u64()?;
        let previous_holes = cursor
            .read_values(previous_holes_len, SIZE_OF_U64, usize::from_bytes)?
            .into_iter()
            .collect();
        Ok((
            change,
            indices.into_iter().zip(values).collect(),
            previous_holes,
        ))
    }

    fn rollback_change(&mut self, change: ChangeData<T>) -> Result<()> {
        ReadWriteRawVec::rollback_change(self, change)
    }

    fn save_change_file(&self, stamp: Stamp, bytes: &[u8]) -> Result<()> {
        self.base.save_change_file(stamp, bytes)
    }

    fn read_current_change_file(&self) -> Result<Vec<u8>> {
        self.base.read_current_change_file()
    }

    fn save_previous(&mut self) {
        self.base.save_prev();
    }

    fn save_previous_for_rollback(&mut self) {
        self.base.save_prev_for_rollback();
    }
}
