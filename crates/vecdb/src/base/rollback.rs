use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::{Bytes, Error, SIZE_OF_U64, Stamp, VecIndex, VecValue};

use super::{ChangeCursor, ChangeData, ReadWriteBaseVec, vec_region_name};

impl<I, T> ReadWriteBaseVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    pub fn changes_path(&self) -> PathBuf {
        self.db_path()
            .join("changes")
            .join(vec_region_name(&self.name, I::to_string()))
    }

    pub fn serialize_changes(
        &self,
        size_of_t: usize,
        collect_stored: impl FnOnce(usize, usize) -> crate::Result<Vec<T>>,
        write_values: impl Fn(&[T], &mut Vec<u8>),
    ) -> crate::Result<Vec<u8>> {
        let prev_stored_len = self.prev_stored_len();
        let stored_len = self.stored_len();
        let truncated = prev_stored_len.saturating_sub(stored_len);

        let value_count = truncated + self.prev_pushed().len() + self.pushed().len();
        let mut bytes = Vec::with_capacity(6 * SIZE_OF_U64 + value_count * size_of_t);

        bytes.extend(self.header.stamp().to_bytes());
        bytes.extend(prev_stored_len.to_bytes());
        bytes.extend(stored_len.to_bytes());
        bytes.extend(truncated.to_bytes());

        if truncated > 0 {
            let truncated_vals = collect_stored(stored_len, prev_stored_len)?;
            write_values(&truncated_vals, &mut bytes);
        }

        bytes.extend(self.prev_pushed().len().to_bytes());
        write_values(self.prev_pushed(), &mut bytes);

        bytes.extend(self.pushed().len().to_bytes());
        write_values(self.pushed(), &mut bytes);

        Ok(bytes)
    }

    /// Returns `Error::Overflow` on arithmetic overflow,
    /// `Error::WrongLength` if the data is truncated.
    pub fn parse_change_data(
        c: &mut ChangeCursor,
        size_of_t: usize,
        read_value: impl Fn(&[u8]) -> crate::Result<T>,
    ) -> crate::Result<ChangeData<T>> {
        let prev_stamp = c.read_stamp()?;
        let prev_stored_len = c.read_u64()?;
        c.skip(SIZE_OF_U64)?; // stored_len, not needed for rollback
        let truncated_count = c.read_u64()?;

        let truncated_start = prev_stored_len
            .checked_sub(truncated_count)
            .ok_or(Error::Underflow)?;
        let truncated_values = c.read_values(truncated_count, size_of_t, &read_value)?;

        let prev_pushed_len = c.read_u64()?;
        let prev_pushed = c.read_values(prev_pushed_len, size_of_t, &read_value)?;

        let pushed_len = c.read_u64()?;
        c.skip(size_of_t.checked_mul(pushed_len).ok_or(Error::Overflow)?)?;

        Ok(ChangeData {
            prev_stamp,
            prev_stored_len,
            truncated_start,
            truncated_values,
            prev_pushed,
        })
    }

    /// Restores the base rollback state. Caller resolves `stored_len` and
    /// `pushed` from the parsed change data according to its own overlay
    /// strategy (raw uses an `updated` map for truncated values, compressed
    /// re-queues them in `pushed`).
    pub fn apply_rollback(&mut self, stamp: Stamp, stored_len: usize, pushed: Vec<T>) {
        self.read_only.header.update_stamp(stamp);
        self.read_only.stored_len.set(stored_len);
        *self.pushed.current_mut() = pushed;
        self.pushed.save();
    }

    /// Caller must check `saved_stamped_changes > 0` before calling.
    pub fn save_change_file(&self, stamp: Stamp, data: &[u8]) -> crate::Result<()> {
        debug_assert!(self.saved_stamped_changes > 0);
        let path = self.changes_path();
        fs::create_dir_all(&path)?;

        let files: BTreeMap<Stamp, PathBuf> = fs::read_dir(&path)?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let s = Stamp::from(path.file_name()?.to_str()?.parse::<u64>().ok()?);
                if s < stamp {
                    Some((s, path))
                } else {
                    let _ = fs::remove_file(&path);
                    None
                }
            })
            .collect();

        let excess = files
            .len()
            .saturating_sub(self.saved_stamped_changes as usize - 1);
        for (_, path) in files.iter().take(excess) {
            fs::remove_file(path)?;
        }

        fs::write(path.join(u64::from(stamp).to_string()), data)?;
        Ok(())
    }

    pub fn save_prev(&mut self) {
        self.previous_stored_len = self.stored_len();
        self.pushed.previous_mut().clear();
    }

    pub fn save_prev_for_rollback(&mut self) {
        self.previous_stored_len = self.stored_len();
        self.pushed.save();
    }

    pub fn read_current_change_file(&self) -> crate::Result<Vec<u8>> {
        let path = self
            .changes_path()
            .join(u64::from(self.header.stamp()).to_string());
        Ok(fs::read(path)?)
    }

    pub fn find_rollback_files(&self) -> crate::Result<BTreeMap<Stamp, PathBuf>> {
        Ok(fs::read_dir(self.changes_path())?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let stamp = Stamp::from(path.file_name()?.to_str()?.parse::<u64>().ok()?);
                Some((stamp, path))
            })
            .collect())
    }
}
