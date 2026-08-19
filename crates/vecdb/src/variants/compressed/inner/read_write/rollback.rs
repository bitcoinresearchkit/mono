use crate::{AnyStoredVec, ChangeCursor, ReadWriteBaseVec, VecIndex, VecValue, WritableVec};

use super::{super::CompressionStrategy, ReadWriteCompressedVec};

impl<I, T, S> ReadWriteCompressedVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    pub(super) fn serialize_compressed_changes(&self) -> crate::Result<Vec<u8>> {
        self.base.serialize_changes(
            Self::SIZE_OF_T,
            |from, to| self.collect_stored_range(from, to),
            |vals, buf| {
                for v in vals {
                    S::write_to_vec(v, buf);
                }
            },
        )
    }

    pub(super) fn deserialize_then_undo_changes(&mut self, bytes: &[u8]) -> crate::Result<()> {
        let mut c = ChangeCursor::new(bytes);
        let change =
            ReadWriteBaseVec::<I, T>::parse_change_data(&mut c, Self::SIZE_OF_T, |b| S::read(b))?;

        // No overlay map: truncated values ride in `pushed` and `stored_len`
        // is clamped to where disk still agrees with the rolled-back state.
        let (stored_len, pushed) = if change.truncated_values.is_empty() {
            (change.prev_stored_len, change.prev_pushed)
        } else {
            let agree_at = change.truncated_start.min(self.real_stored_len());
            let mut buf = change.truncated_values;
            buf.extend(change.prev_pushed);
            (agree_at, buf)
        };
        self.base
            .apply_rollback(change.prev_stamp, stored_len, pushed);

        Ok(())
    }
}
