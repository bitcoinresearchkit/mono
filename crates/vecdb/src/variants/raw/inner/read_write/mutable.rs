//! In-place updates and their undo records, for the `MutableVec` wrapping this vec.

use std::collections::{BTreeMap, BTreeSet};

use super::{super::RawStrategy, ReadWriteRawVec};
use crate::{
    AnyStoredVec, Bytes, ChangeCursor, HEADER_OFFSET, ReadWriteBaseVec, Result, SIZE_OF_U64,
    VecIndex, VecValue, cache::CachePolicy,
};

impl<I, T, S, C: CachePolicy> ReadWriteRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    /// Overwrites stored values at their indexes.
    pub(crate) fn write_updates(&self, updated: BTreeMap<usize, T>) -> Result<()> {
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
    pub(crate) fn append_previous_values(
        &self,
        indices: &[usize],
        previous: &BTreeMap<usize, T>,
        bytes: &mut Vec<u8>,
    ) {
        let reader = self.reader();
        for &index in indices {
            match previous.get(&index) {
                Some(value) => S::write_to_vec(value, bytes),
                None => S::write_to_vec(&reader.get_at(index), bytes),
            }
        }
    }

    /// Parses a mutable change file: the base change data, then the previous values of updated
    /// indexes, then the previous holes.
    pub(crate) fn parse_mutable_changes(
        bytes: &[u8],
    ) -> Result<(Vec<(usize, T)>, BTreeSet<usize>)> {
        let mut cursor = ChangeCursor::new(bytes);
        ReadWriteBaseVec::<I, T>::parse_change_data::<S>(&mut cursor)?;
        let modified_len = cursor.read_u64()?;
        let indices = cursor.read_values(modified_len, SIZE_OF_U64, usize::from_bytes)?;
        let values = cursor.read_values(modified_len, Self::SIZE_OF_T, S::read)?;
        let previous_holes_len = cursor.read_u64()?;
        let previous_holes = cursor
            .read_values(previous_holes_len, SIZE_OF_U64, usize::from_bytes)?
            .into_iter()
            .collect();
        Ok((indices.into_iter().zip(values).collect(), previous_holes))
    }
}
