use std::collections::BTreeSet;

use crate::{
    AnyStoredVec, Bytes, ChangeCursor, ChangeData, ReadWriteBaseVec, SIZE_OF_U64, VecIndex,
    VecValue,
};

use super::{super::RawStrategy, ReadWriteRawVec, change::RawChangeData};

impl<I, T, S> ReadWriteRawVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    pub(super) fn serialize_raw_changes(&self) -> crate::Result<Vec<u8>> {
        let mut bytes = self.base.serialize_changes(
            Self::SIZE_OF_T,
            |from, to| self.collect_stored_range(from, to),
            |vals, buf| {
                for v in vals {
                    S::write_to_vec(v, buf);
                }
            },
        )?;

        let reader = self.raw_reader();
        let updated = self.updated();
        let prev_updated = self.prev_updated();

        // Collect all indices that need change tracking: entries currently modified
        // AND entries that were in prev_updated but removed (e.g., by delete_at).
        // Without the latter, rollback after rollback loses track of deleted entries.
        let all_keys: BTreeSet<usize> =
            updated.keys().chain(prev_updated.keys()).copied().collect();

        bytes.extend(all_keys.len().to_bytes());
        for &i in &all_keys {
            bytes.extend(i.to_bytes());
        }
        for &i in &all_keys {
            if let Some(v) = prev_updated.get(&i) {
                S::write_to_vec(v, &mut bytes);
            } else {
                S::write_to_vec(&self.unchecked_read_at(i, &reader), &mut bytes);
            }
        }

        let prev_holes = self.prev_holes();
        bytes.extend(prev_holes.len().to_bytes());
        for &hole in prev_holes {
            bytes.extend(hole.to_bytes());
        }

        Ok(bytes)
    }

    fn parse_raw_change_data(bytes: &[u8]) -> crate::Result<RawChangeData<T>> {
        let mut c = ChangeCursor::new(bytes);
        let base =
            ReadWriteBaseVec::<I, T>::parse_change_data(&mut c, Self::SIZE_OF_T, |b| S::read(b))?;

        let modified_len = c.read_u64()?;
        let indices = c.read_values(modified_len, SIZE_OF_U64, usize::from_bytes)?;
        let values = c.read_values(modified_len, Self::SIZE_OF_T, |b| S::read(b))?;
        let modifications = indices.into_iter().zip(values).collect();

        let prev_holes_len = c.read_u64()?;
        let prev_holes = c
            .read_values(prev_holes_len, SIZE_OF_U64, usize::from_bytes)?
            .into_iter()
            .collect();

        Ok(RawChangeData {
            base,
            modifications,
            prev_holes,
        })
    }

    pub(super) fn deserialize_then_undo_changes(&mut self, bytes: &[u8]) -> crate::Result<()> {
        let RawChangeData {
            base:
                ChangeData {
                    prev_stamp,
                    prev_stored_len,
                    truncated_start,
                    truncated_values,
                    prev_pushed,
                    ..
                },
            modifications,
            prev_holes,
        } = Self::parse_raw_change_data(bytes)?;

        // Only needed when the rolled-back flush appended (prev_stored_len <
        // current): any holes/updated in the now-gone range must be dropped.
        if prev_stored_len < self.stored_len() {
            self.truncate_dirty_at(prev_stored_len);
        }

        // Truncated values overlay via `updated` at indices beyond the
        // current on-disk length; the next write() extends the region.
        self.base
            .apply_rollback(prev_stamp, prev_stored_len, prev_pushed);
        for (i, val) in truncated_values.into_iter().enumerate() {
            self.mut_updated().insert(truncated_start + i, val);
        }

        for (idx, val) in modifications {
            self.update_at(idx, val)?;
        }

        if !prev_holes.is_empty() || !self.holes().is_empty() || !self.prev_holes().is_empty() {
            *self.holes.current_mut() = prev_holes;
            self.holes.save();
        }

        self.updated.save();

        Ok(())
    }
}
