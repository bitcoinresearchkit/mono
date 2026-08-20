use crate::{AnyStoredVec, ChangeCursor, ReadWriteBaseVec, VecIndex, VecValue};

use super::{super::RawStrategy, ReadWriteRawVec};

impl<I, T, S> ReadWriteRawVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    pub(super) fn serialize_raw_changes(&self) -> crate::Result<Vec<u8>> {
        self.base.serialize_changes(
            Self::SIZE_OF_T,
            |from, to| self.collect_stored_range(from, to),
            |values, bytes| {
                for value in values {
                    S::write_to_vec(value, bytes);
                }
            },
        )
    }

    pub(super) fn deserialize_then_undo_changes(&mut self, bytes: &[u8]) -> crate::Result<()> {
        let mut cursor = ChangeCursor::new(bytes);
        let change =
            ReadWriteBaseVec::<I, T>::parse_change_data(&mut cursor, Self::SIZE_OF_T, S::read)?;

        let (stored_len, pushed) = if change.truncated_values.is_empty() {
            (change.prev_stored_len, change.prev_pushed)
        } else {
            let agree_at = change.truncated_start.min(self.real_stored_len());
            let mut pushed = change.truncated_values;
            pushed.extend(change.prev_pushed);
            (agree_at, pushed)
        };
        self.base
            .apply_rollback(change.prev_stamp, stored_len, pushed);

        Ok(())
    }
}
